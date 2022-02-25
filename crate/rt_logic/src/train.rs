use std::{fmt, marker::PhantomData};

use choochoo_cfg_model::{
    indicatif::MultiProgress,
    rt::{OpStatus, ResourceIds, StationMutRef, StationRtId, TrainResources},
};
use choochoo_rt_model::{
    error::StationSpecError, Destination, EnsureOutcomeErr, EnsureOutcomeOk, Error,
};
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{Driver, OpStatusUpdater, ResourceInitializer};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train<E>(PhantomData<E>);

impl<E> Train<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    /// Ensures the given destination is reached.
    pub async fn reach(dest: &mut Destination<E>) -> Result<TrainResources<E>, Error<E>> {
        let progress_fut = Self::progress_tracker_init(dest);

        let mut train_resources = TrainResources::new();
        if dest.station_specs().node_count() == 0 {
            return Ok(train_resources);
        }

        ResourceInitializer::initialize(dest, &mut train_resources).await?;

        train_resources = Self::stations_setup(dest, train_resources)
            .await
            .or_else(|error| {
                if let Error::StationSetup { train_resources } = error {
                    Ok(train_resources)
                } else {
                    Err(error)
                }
            })?;

        if train_resources.station_errors().read().await.is_empty() {
            train_resources = Self::stations_visit(dest, train_resources).await?;
        }

        Self::progress_tracker_join(dest, progress_fut).await?;

        Ok(train_resources)
    }

    /// Initializes the progress tracker.
    fn progress_tracker_init(dest: &Destination<E>) -> JoinHandle<std::io::Result<()>> {
        let multi_progress = MultiProgress::new();
        dest.station_specs()
            .graph()
            .node_indices()
            .filter_map(|station_rt_id| dest.station_progresses().get(&station_rt_id))
            .for_each(|station_progress| {
                let progress_bar = station_progress.borrow().progress_bar().clone();
                let progress_bar_for_tick = progress_bar.clone();
                multi_progress.add(progress_bar);

                // Needed to render all progress bars.
                progress_bar_for_tick.tick();
            });

        tokio::task::spawn_blocking(move || multi_progress.join())
    }

    /// Waits for the progress tracker to complete.
    async fn progress_tracker_join(
        dest: &mut Destination<E>,
        progress_fut: JoinHandle<Result<(), std::io::Error>>,
    ) -> Result<(), Error<E>> {
        // We need to finish / abandon all progress bars, otherwise the `MultiProgress`
        // will never finish.
        dest.stations_mut().for_each(|station| {
            if !station.progress.progress_bar().is_finished() {
                station.progress.progress_bar().finish_at_current_pos();
            }
        });

        progress_fut
            .await
            .map_err(Error::MultiProgressTaskJoin)?
            .map_err(Error::MultiProgressJoin)?;

        Ok(())
    }

    async fn stations_setup(
        dest: &mut Destination<E>,
        train_resources: TrainResources<E>,
    ) -> Result<TrainResources<E>, Error<E>> {
        stream::iter(dest.stations_mut().map(Result::<_, Error<E>>::Ok))
            .try_fold(
                train_resources,
                |mut train_resources, mut station| async move {
                    let setup_result = station.setup(&mut train_resources).await;

                    match setup_result {
                        Ok(progress_limit) => {
                            station.progress.op_status = OpStatus::SetupSuccess;
                            station.progress.progress_limit_set(progress_limit);
                            station.progress.progress_style_update();
                            Ok(train_resources)
                        }
                        Err(station_error) => {
                            station.progress.op_status = OpStatus::SetupFail;
                            Self::station_error_insert(
                                &train_resources,
                                station.rt_id,
                                station_error,
                            )
                            .await;
                            station.progress.progress_style_update();
                            Err(Error::StationSetup { train_resources })
                        }
                    }
                },
            )
            .await
    }

    async fn stations_visit(
        dest: &mut Destination<E>,
        train_resources: TrainResources<E>,
    ) -> Result<TrainResources<E>, Error<E>> {
        let dest = &dest;

        // Set `ParentPending` stations to `OpQueued` if they have no dependencies.
        OpStatusUpdater::update(dest);

        let resources = &train_resources;
        let (resource_ids_tx, mut resource_ids_rx) = mpsc::unbounded_channel::<ResourceIds>();
        let resource_ids_tx = &resource_ids_tx;
        dest.stations_mut_stream()
            .map(Result::<_, Error<E>>::Ok)
            .map_ok(|mut station| async move {
                station.progress.progress_style_update();
                let resource_ids = if station.progress.op_status == OpStatus::OpQueued
                    || station.progress.op_status == OpStatus::SetupSuccess
                {
                    // Because this is in an async block, concurrent tasks may access this
                    // station's `op_status` while the `visit()` is
                    // `await`ed.
                    station.progress.op_status = OpStatus::WorkInProgress;
                    station.progress.progress_style_update();

                    Self::stations_visit_ensure(&mut station, resources).await
                } else {
                    None
                };
                station.progress.progress_style_update();

                let resource_ids_result = resource_ids.map(|resource_ids| {
                    resource_ids_tx.send(resource_ids).map_err(|error| {
                        Error::ResourceIdsChannelClosed {
                            station_id: station.spec.id().clone(),
                            error,
                        }
                    })
                });

                (station.rt_id, resource_ids_result)
            })
            .try_for_each_concurrent(4, |station_rt_id_and_resource_ids_result| async {
                let (station_rt_id, resource_ids_result) =
                    station_rt_id_and_resource_ids_result.await;

                OpStatusUpdater::update_children(dest, station_rt_id);
                resource_ids_result.unwrap_or_else(|| Result::Ok(()))
            })
            .await?;
        drop(resource_ids_tx);

        resource_ids_rx.close();
        let resource_ids_all = stream::poll_fn(|ctx| resource_ids_rx.poll_recv(ctx))
            .fold(
                ResourceIds::new(),
                |mut resource_ids_all, resource_ids_current| async move {
                    resource_ids_all.extend(resource_ids_current.0.into_iter());
                    resource_ids_all
                },
            )
            .await;

        Ok(train_resources)
    }

    async fn stations_visit_ensure(
        station: &mut StationMutRef<'_, E>,
        report: &TrainResources<E>,
    ) -> Option<ResourceIds> {
        match Driver::ensure(station, report).await {
            Ok(EnsureOutcomeOk::Changed {
                resource_ids,
                station_spec_error,
            }) => {
                station.progress.op_status = OpStatus::WorkSuccess;

                if let Some(station_spec_error) = station_spec_error {
                    let station_error = E::from(station_spec_error);

                    Self::station_error_insert(report, station.rt_id, station_error).await;
                }

                Some(resource_ids)
            }
            Ok(EnsureOutcomeOk::Unchanged) => {
                station.progress.op_status = OpStatus::WorkUnnecessary;
                None
            }
            Err(EnsureOutcomeErr::CheckBorrowFail(_borrow_fail)) => {
                station.progress.op_status = OpStatus::CheckFail;

                // TODO: insert borrow fail error somewhere

                None
            }
            Err(EnsureOutcomeErr::CheckFail(station_error)) => {
                station.progress.op_status = OpStatus::CheckFail;

                Self::station_error_insert(report, station.rt_id, station_error).await;

                None
            }
            Err(EnsureOutcomeErr::VisitBorrowFail(_borrow_fail)) => {
                station.progress.op_status = OpStatus::WorkFail;

                // TODO: insert borrow fail error somewhere

                None
            }
            Err(EnsureOutcomeErr::WorkFail {
                resource_ids,
                error: station_error,
            }) => {
                station.progress.op_status = OpStatus::WorkFail;

                Self::station_error_insert(report, station.rt_id, station_error).await;
                Some(resource_ids)
            }
        }
    }

    async fn station_error_insert(
        train_resources: &TrainResources<E>,
        station_rt_id: StationRtId,
        station_error: E,
    ) {
        let station_errors = train_resources.station_errors();
        let mut station_errors = station_errors.write().await;
        station_errors.insert(station_rt_id, station_error);
    }
}

use std::{fmt, marker::PhantomData, num::NonZeroUsize};

use choochoo_cfg_model::{
    indicatif::MultiProgress,
    rt::{OpStatus, ResIds, StationMutRef, StationRtId, TrainResources},
    StationSpecs,
};
use choochoo_resource::ProfileHistoryDir;
use choochoo_rt_model::{
    error::StationSpecError, Destination, EnsureOutcomeErr, EnsureOutcomeOk, Error, TrainReport,
};
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver},
    task::JoinHandle,
};

use crate::{Driver, OpStatusUpdater, ResIdPersister, ResourceInitializer};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train<E> {
    /// Maximum number of stations to run concurrently.
    concurrency_max: Option<NonZeroUsize>,
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> Train<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    /// Returns a `Train` to visit stations.
    ///
    /// The `concurrency_max` parameter is used as follows:
    ///
    /// * `None`: There is no concurrency limit.
    ///
    ///     This is the same as using [`Train::default`] to initialize a train.
    ///
    /// * `Some(n)`: At most `n` stations will be visited concurrently.
    ///
    ///     You may wish to use the [`std::thread::available_parallelism`]
    ///     function which  usually corresponds to the amount of CPUs or
    ///     processors a computer has.
    ///
    /// # Parameters
    ///
    /// * `concurrency_max`: Maximum number of stations to visit concurrently.
    fn new(concurrency_max: Option<NonZeroUsize>) -> Self {
        Self {
            concurrency_max,
            marker: PhantomData,
        }
    }

    /// Ensures the given destination is reached.
    pub async fn reach(&self, dest: &mut Destination<E>) -> Result<TrainReport<E>, Error<E>> {
        let progress_fut = Self::progress_tracker_init(dest);

        if dest.station_specs().node_count() == 0 {
            Self::progress_tracker_join(dest, progress_fut).await?;
            return Ok(TrainReport::default());
        }

        let mut train_resources = TrainResources::new();
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

        // If here are no errors during setup, then we visit each station.
        let train_report = if train_resources.station_errors().read().await.is_empty() {
            let train_report = self.stations_visit(dest, train_resources).await?;
            Self::progress_tracker_join(dest, progress_fut).await?;
            train_report
        } else {
            Self::progress_tracker_join(dest, progress_fut).await?;
            TrainReport::new(train_resources, ResIds::new())
        };

        Ok(train_report)
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
                            station.progress.progress_style_update();
                            Self::station_error_insert(
                                &train_resources,
                                station.rt_id,
                                station_error,
                            )
                            .await;
                            Err(Error::StationSetup { train_resources })
                        }
                    }
                },
            )
            .await
    }

    async fn stations_visit(
        &self,
        dest: &mut Destination<E>,
        train_resources: TrainResources<E>,
    ) -> Result<TrainReport<E>, Error<E>> {
        // Set `ParentPending` stations to `OpQueued` if they have no dependencies.
        OpStatusUpdater::update(dest);

        let (res_ids_tx, mut res_ids_rx) = mpsc::unbounded_channel::<(StationRtId, ResIds)>();
        self.stations_visit_each(dest, &train_resources, res_ids_tx)
            .await?;

        res_ids_rx.close();

        let profile_history_dir = train_resources.borrow::<ProfileHistoryDir>();
        let res_ids = Self::stations_visit_res_ids_wait(
            dest.station_specs(),
            &profile_history_dir,
            res_ids_rx,
        )
        .await?;
        drop(profile_history_dir);

        let train_report = TrainReport::new(train_resources, res_ids);
        Ok(train_report)
    }

    async fn stations_visit_each(
        &self,
        dest: &mut Destination<E>,
        train_resources: &TrainResources<E>,
        res_ids_tx: mpsc::UnboundedSender<(StationRtId, ResIds)>,
    ) -> Result<(), Error<E>> {
        let res_ids_tx_ref = &res_ids_tx;
        dest.stations_mut_stream()
            .map(Result::<_, Error<E>>::Ok)
            .map_ok(|mut station| async move {
                station.progress.progress_style_update();
                let res_ids = if station.progress.op_status == OpStatus::OpQueued
                    || station.progress.op_status == OpStatus::SetupSuccess
                {
                    // Because this is in an async block, concurrent tasks may access this
                    // station's `op_status` while the `visit()` is
                    // `await`ed.
                    station.progress.op_status = OpStatus::WorkInProgress;
                    station.progress.progress_style_update();

                    Self::stations_visit_station_ensure(&mut station, train_resources).await
                } else {
                    None
                };
                station.progress.progress_style_update();

                let res_ids_result = res_ids.map(|res_ids| {
                    res_ids_tx_ref
                        .send((station.rt_id, res_ids))
                        .map_err(|error| Error::ResIdsChannelClosed {
                            station_id: station.spec.id().clone(),
                            error,
                        })
                });

                (station.rt_id, res_ids_result)
            })
            .try_for_each_concurrent(
                self.concurrency_max.map(NonZeroUsize::get),
                |station_rt_id_and_res_ids_result| async {
                    let (station_rt_id, res_ids_result) = station_rt_id_and_res_ids_result.await;

                    OpStatusUpdater::update_children(dest, station_rt_id);
                    res_ids_result.unwrap_or(Result::Ok(()))
                },
            )
            .await?;
        drop(res_ids_tx);
        Ok(())
    }

    async fn stations_visit_station_ensure(
        station: &mut StationMutRef<'_, E>,
        train_resources: &TrainResources<E>,
    ) -> Option<ResIds> {
        match Driver::ensure(station, train_resources).await {
            Ok(EnsureOutcomeOk::Changed {
                res_ids,
                station_spec_error,
            }) => {
                station.progress.op_status = OpStatus::WorkSuccess;

                if let Some(station_spec_error) = station_spec_error {
                    let station_error = E::from(station_spec_error);

                    Self::station_error_insert(train_resources, station.rt_id, station_error).await;
                }

                Some(res_ids)
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

                Self::station_error_insert(train_resources, station.rt_id, station_error).await;

                None
            }
            Err(EnsureOutcomeErr::VisitBorrowFail(_borrow_fail)) => {
                station.progress.op_status = OpStatus::WorkFail;

                // TODO: insert borrow fail error somewhere

                None
            }
            Err(EnsureOutcomeErr::WorkFail {
                res_ids,
                error: station_error,
            }) => {
                station.progress.op_status = OpStatus::WorkFail;

                Self::station_error_insert(train_resources, station.rt_id, station_error).await;
                Some(res_ids)
            }
        }
    }

    async fn stations_visit_res_ids_wait(
        station_specs: &StationSpecs<E>,
        profile_history_dir: &ProfileHistoryDir,
        mut res_ids_rx: UnboundedReceiver<(StationRtId, ResIds)>,
    ) -> Result<ResIds, Error<E>> {
        let res_ids = stream::poll_fn(|ctx| res_ids_rx.poll_recv(ctx))
            .map(Result::<_, Error<E>>::Ok)
            .and_then(|(station_rt_id, res_ids_current)| async move {
                let station_id = station_specs[station_rt_id].id();
                ResIdPersister::<E>::persist(profile_history_dir, station_id, &res_ids_current)
                    .await?;
                Ok(res_ids_current)
            })
            .try_fold(
                ResIds::new(),
                |mut res_ids_all, mut res_ids_current| async move {
                    res_ids_all.extend(res_ids_current.drain(..));

                    Ok(res_ids_all)
                },
            )
            .await?;

        Ok(res_ids)
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

impl<E> Default for Train<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new(None)
    }
}

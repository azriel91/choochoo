use std::{fmt, marker::PhantomData, num::NonZeroUsize};

use choochoo_cfg_model::{
    rt::{OpStatus, ResIds, StationMutRef, StationRtId, TrainResources},
    StationSpecs,
};
use choochoo_resource::ProfileHistoryDir;
use choochoo_rt_model::{
    error::StationSpecError, CreateEnsureOutcomeErr, CreateEnsureOutcomeOk, Destination, Error,
    TrainReport,
};
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{CreateDriver, OpStatusUpdater, ResIdPersister, Train};

/// Logic to manage resource creation.
pub(crate) struct TrainCreate<E>(PhantomData<E>);

impl<E> TrainCreate<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    /// Runs the `create` functions for each station.
    pub(crate) async fn stations_visit(
        train: &Train<E>,
        dest: &mut Destination<E>,
        train_resources: TrainResources<E>,
    ) -> Result<TrainReport<E>, Error<E>> {
        // Set `ParentPending` stations to `OpQueued` if they have no dependencies.
        OpStatusUpdater::update(dest);

        let (res_ids_tx, res_ids_rx) = mpsc::unbounded_channel::<(StationRtId, ResIds)>();
        let stations_visit_each =
            Self::stations_visit_each(train, dest, &train_resources, res_ids_tx);

        let profile_history_dir = train_resources.borrow::<ProfileHistoryDir>();
        let stations_visit_res_ids_wait = Self::stations_visit_res_ids_wait(
            dest.station_specs(),
            &profile_history_dir,
            res_ids_rx,
        );

        let ((), res_ids) = futures::try_join!(stations_visit_each, stations_visit_res_ids_wait)?;
        drop(profile_history_dir);

        let train_report = TrainReport::new(train_resources, res_ids);
        Ok(train_report)
    }

    async fn stations_visit_each(
        train: &Train<E>,
        dest: &Destination<E>,
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
                train.concurrency_max.map(NonZeroUsize::get),
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
        match CreateDriver::ensure(station, train_resources).await {
            Ok(CreateEnsureOutcomeOk::Changed {
                res_ids,
                station_spec_error,
            }) => {
                station.progress.op_status = OpStatus::WorkSuccess;

                if let Some(station_spec_error) = station_spec_error {
                    let station_error = E::from(station_spec_error);

                    Train::station_error_insert(train_resources, station.rt_id, station_error)
                        .await;
                }

                Some(res_ids)
            }
            Ok(CreateEnsureOutcomeOk::Unchanged) => {
                station.progress.op_status = OpStatus::WorkUnnecessary;
                None
            }
            Err(CreateEnsureOutcomeErr::CheckBorrowFail(_borrow_fail)) => {
                station.progress.op_status = OpStatus::CheckFail;

                // TODO: insert borrow fail error somewhere

                None
            }
            Err(CreateEnsureOutcomeErr::CheckFail(station_error)) => {
                station.progress.op_status = OpStatus::CheckFail;

                Train::station_error_insert(train_resources, station.rt_id, station_error).await;

                None
            }
            Err(CreateEnsureOutcomeErr::VisitBorrowFail(_borrow_fail)) => {
                station.progress.op_status = OpStatus::WorkFail;

                // TODO: insert borrow fail error somewhere

                None
            }
            Err(CreateEnsureOutcomeErr::WorkFail {
                res_ids,
                error: station_error,
            }) => {
                station.progress.op_status = OpStatus::WorkFail;

                Train::station_error_insert(train_resources, station.rt_id, station_error).await;
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
}

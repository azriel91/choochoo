use std::{fmt, marker::PhantomData, num::NonZeroUsize};

use choochoo_cfg_model::rt::{OpStatus, ResIds, StationMutRef, TrainResources};
use choochoo_rt_model::{
    error::StationSpecError, CleanEnsureOutcomeErr, CleanEnsureOutcomeOk, Destination, Error,
    TrainReport,
};
use futures::stream::StreamExt;

use crate::{CleanDriver, CleanOpStatusUpdater, Train};

/// Logic to manage resource cleaning.
pub(crate) struct TrainClean<E>(PhantomData<E>);

impl<E> TrainClean<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    /// Runs the `clean` functions for each station.
    pub(crate) async fn stations_visit(
        train: &Train<E>,
        dest: &mut Destination<E>,
        train_resources: TrainResources<E>,
    ) -> Result<TrainReport<E>, Error<E>> {
        // Set `ParentPending` stations to `OpQueued` if they have no dependencies.
        CleanOpStatusUpdater::update(dest);

        Self::stations_visit_each(train, dest, &train_resources).await;

        let train_report = TrainReport::new(train_resources, ResIds::new());
        Ok(train_report)
    }

    async fn stations_visit_each(
        train: &Train<E>,
        dest: &Destination<E>,
        train_resources: &TrainResources<E>,
    ) {
        dest.stations_mut_stream_rev()
            .map(|mut station| async move {
                station.progress.progress_style_update();
                if station.progress.op_status == OpStatus::OpQueued
                    || station.progress.op_status == OpStatus::SetupSuccess
                {
                    // Because this is in an async block, concurrent tasks may access this
                    // station's `op_status` while the `visit()` is
                    // `await`ed.
                    station.progress.op_status = OpStatus::WorkInProgress;
                    station.progress.progress_style_update();

                    Self::stations_visit_station_ensure(&mut station, train_resources).await
                };
                station.progress.progress_style_update();

                station.rt_id
            })
            .for_each_concurrent(
                train.concurrency_max.map(NonZeroUsize::get),
                |station_rt_id| async {
                    CleanOpStatusUpdater::update_successors(dest, station_rt_id.await);
                },
            )
            .await;
    }

    async fn stations_visit_station_ensure(
        station: &mut StationMutRef<'_, E>,
        train_resources: &TrainResources<E>,
    ) {
        eprintln!("{}", station.spec.id());
        match dbg!(CleanDriver::ensure(station, train_resources).await) {
            Ok(CleanEnsureOutcomeOk::NothingToDo) => {
                station.progress.op_status = OpStatus::WorkUnnecessary;
            }
            Ok(CleanEnsureOutcomeOk::Changed { station_spec_error }) => {
                station.progress.op_status = OpStatus::WorkSuccess;

                if let Some(station_spec_error) = station_spec_error {
                    let station_error = E::from(station_spec_error);

                    Train::station_error_insert(train_resources, station.rt_id, station_error)
                        .await;
                }
            }
            Ok(CleanEnsureOutcomeOk::Unchanged) => {
                station.progress.op_status = OpStatus::WorkUnnecessary;
            }
            Err(CleanEnsureOutcomeErr::Never) => {
                unreachable!("CleanEnsureOutcomeErr::Never should never be reached");
            }
            Err(CleanEnsureOutcomeErr::CheckBorrowFail(_borrow_fail)) => {
                station.progress.op_status = OpStatus::CheckFail;

                // TODO: insert borrow fail error somewhere
            }
            Err(CleanEnsureOutcomeErr::CheckFail(station_error)) => {
                station.progress.op_status = OpStatus::CheckFail;

                Train::station_error_insert(train_resources, station.rt_id, station_error).await;
            }
            Err(CleanEnsureOutcomeErr::VisitBorrowFail(_borrow_fail)) => {
                station.progress.op_status = OpStatus::WorkFail;

                // TODO: insert borrow fail error somewhere
            }
            Err(CleanEnsureOutcomeErr::WorkFail {
                error: station_error,
            }) => {
                station.progress.op_status = OpStatus::WorkFail;

                Train::station_error_insert(train_resources, station.rt_id, station_error).await;
            }
        }
    }
}

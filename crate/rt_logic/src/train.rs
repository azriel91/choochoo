use std::{fmt, marker::PhantomData};

use choochoo_cfg_model::{
    indicatif::MultiProgress,
    rt::{StationRtId, TrainReport, VisitStatus},
};
use choochoo_rt_model::{
    error::StationSpecError, Destination, EnsureOutcomeErr, EnsureOutcomeOk, Error,
};
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio::task::JoinHandle;

use crate::{Driver, VisitStatusUpdater};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train<E>(PhantomData<E>);

impl<E> Train<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    /// Ensures the given destination is reached.
    pub async fn reach(dest: &mut Destination<E>) -> Result<TrainReport<E>, Error<E>> {
        let progress_fut = Self::progress_tracker_init(dest);

        let mut train_report = TrainReport::new();
        if dest.station_specs().node_count() == 0 {
            return Ok(train_report);
        }

        train_report = Self::stations_setup(dest, train_report)
            .await
            .or_else(|error| {
                if let Error::StationSetup { train_report } = error {
                    Ok(train_report)
                } else {
                    Err(error)
                }
            })?;

        if train_report.station_errors().read().await.is_empty() {
            train_report = Self::stations_visit(dest, train_report).await;
        }

        Self::progress_tracker_join(dest, progress_fut).await?;

        Ok(train_report)
    }

    /// Initializes the progress tracker.
    fn progress_tracker_init(dest: &Destination<E>) -> JoinHandle<std::io::Result<()>> {
        let multi_progress = MultiProgress::new();
        multi_progress.set_draw_target(choochoo_cfg_model::indicatif::ProgressDrawTarget::hidden());
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
        train_report: TrainReport<E>,
    ) -> Result<TrainReport<E>, Error<E>> {
        stream::iter(dest.stations_mut().map(Result::<_, Error<E>>::Ok))
            .try_fold(train_report, |mut train_report, mut station| async move {
                let setup_result = station.setup(&mut train_report).await;

                match setup_result {
                    Ok(progress_limit) => {
                        station.progress.visit_status = VisitStatus::SetupSuccess;
                        station.progress.progress_limit_set(progress_limit);
                        station.progress.progress_style_update();
                        Ok(train_report)
                    }
                    Err(station_error) => {
                        station.progress.visit_status = VisitStatus::SetupFail;
                        Self::station_error_insert(&train_report, station.rt_id, station_error)
                            .await;
                        station.progress.progress_style_update();
                        Err(Error::StationSetup { train_report })
                    }
                }
            })
            .await
    }

    async fn stations_visit(
        dest: &mut Destination<E>,
        train_report: TrainReport<E>,
    ) -> TrainReport<E> {
        let dest = &dest;

        // Set `ParentPending` stations to `VisitQueued` if they have no dependencies.
        VisitStatusUpdater::update(dest);

        let report = &train_report;
        dest.stations_mut_stream()
            .for_each_concurrent(4, |mut station| {
                if let Some(visit_status) =
                    VisitStatusUpdater::visit_status_next(dest, station.rt_id)
                {
                    station.progress.visit_status = dbg!(visit_status);
                    station.progress.progress_style_update();
                }

                async move {
                    if dbg!(station.progress.visit_status) == VisitStatus::VisitQueued
                        || station.progress.visit_status == VisitStatus::SetupSuccess
                    {
                        // Because this is in an async block, concurrent tasks may access this
                        // station's `visit_status` while the `visit()` is
                        // `await`ed.
                        station.progress.visit_status = VisitStatus::InProgress;
                        station.progress.progress_style_update();

                        match Driver::ensure(&mut station, report).await {
                            Ok(EnsureOutcomeOk::Changed { station_spec_error }) => {
                                station.progress.visit_status = VisitStatus::VisitSuccess;

                                if let Some(station_spec_error) = station_spec_error {
                                    let station_error = E::from(station_spec_error);

                                    Self::station_error_insert(
                                        report,
                                        station.rt_id,
                                        station_error,
                                    )
                                    .await;
                                }
                            }
                            Ok(EnsureOutcomeOk::Unchanged) => {
                                station.progress.visit_status = VisitStatus::VisitUnnecessary;
                            }
                            Err(EnsureOutcomeErr::CheckBorrowFail(_borrow_fail)) => {
                                station.progress.visit_status = VisitStatus::CheckFail;

                                // TODO: insert borrow fail error somewhere
                            }
                            Err(EnsureOutcomeErr::CheckFail(station_error)) => {
                                station.progress.visit_status = VisitStatus::CheckFail;

                                Self::station_error_insert(report, station.rt_id, station_error)
                                    .await;
                            }
                            Err(EnsureOutcomeErr::VisitBorrowFail(_borrow_fail)) => {
                                station.progress.visit_status = VisitStatus::VisitFail;

                                // TODO: insert borrow fail error somewhere
                            }
                            Err(EnsureOutcomeErr::VisitFail(station_error)) => {
                                station.progress.visit_status = VisitStatus::VisitFail;

                                Self::station_error_insert(report, station.rt_id, station_error)
                                    .await;
                            }
                        }
                    }
                }
            })
            .await;

        train_report
    }

    async fn station_error_insert(
        train_report: &TrainReport<E>,
        station_rt_id: StationRtId,
        station_error: E,
    ) {
        let station_errors = train_report.station_errors();
        let mut station_errors = station_errors.write().await;
        station_errors.insert(station_rt_id, station_error);
    }
}

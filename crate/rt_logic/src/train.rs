use std::{fmt, marker::PhantomData};

use choochoo_cfg_model::{indicatif::MultiProgress, VisitStatus};
use choochoo_rt_model::{
    error::StationSpecError, Destination, EnsureOutcomeErr, EnsureOutcomeOk, Error, StationMut,
    TrainReport,
};
use tokio::task::JoinHandle;

use crate::{strategy::IntegrityStrat, Driver, VisitStatusUpdater};

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

        let train_report = Self::stations_visit(dest).await?;

        Self::progress_tracker_join(dest, progress_fut).await?;

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

    async fn stations_visit(dest: &mut Destination<E>) -> Result<TrainReport<E>, Error<E>> {
        // Set `ParentPending` stations to `Queued` if they have no dependencies.
        VisitStatusUpdater::update(dest);

        IntegrityStrat::iter(dest, TrainReport::new(), |mut station, train_report| {
            Box::pin(async move {
                // Because this is in an async block, concurrent tasks may access this station's
                // `visit_status` while the `visit()` is `await`ed.
                station.progress.visit_status = VisitStatus::InProgress;

                match Driver::ensure(&mut station, train_report).await {
                    Ok(EnsureOutcomeOk::Changed { station_spec_error }) => {
                        station.progress.visit_status = VisitStatus::VisitSuccess;

                        if let Some(station_spec_error) = station_spec_error {
                            let station_error = E::from(station_spec_error);

                            Self::station_error_insert(train_report, station, station_error).await;
                        }
                    }
                    Ok(EnsureOutcomeOk::Unchanged) => {
                        station.progress.visit_status = VisitStatus::VisitUnnecessary;
                    }
                    Err(EnsureOutcomeErr::CheckFail(station_error)) => {
                        station.progress.visit_status = VisitStatus::CheckFail;

                        Self::station_error_insert(train_report, station, station_error).await;
                    }
                    Err(EnsureOutcomeErr::VisitFail(station_error)) => {
                        station.progress.visit_status = VisitStatus::VisitFail;

                        Self::station_error_insert(train_report, station, station_error).await;
                    }
                }

                train_report
            })
        })
        .await
    }

    async fn station_error_insert(
        train_report: &TrainReport<E>,
        station: &StationMut<'_, E>,
        station_error: E,
    ) {
        let station_errors = train_report.station_errors();
        let mut station_errors = station_errors.write().await;
        station_errors.insert(station.rt_id, station_error);
    }
}

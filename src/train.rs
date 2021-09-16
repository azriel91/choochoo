use std::{fmt, marker::PhantomData};

use tokio::task::JoinHandle;

use crate::{
    cfg_model::{indicatif::MultiProgress, VisitStatus},
    rt_logic::{strategy::IntegrityStrat, Driver, VisitStatusUpdater},
    rt_model::{
        error::StationSpecError, Destination, EnsureOutcomeErr, EnsureOutcomeOk, Error, TrainReport,
    },
};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train<E>(PhantomData<E>);

impl<E> Train<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    /// Ensures the given destination is reached.
    pub async fn reach(dest: &mut Destination<E>) -> Result<TrainReport<E>, Error<E>> {
        let progress_fut = Self::progress_tracker_init(&dest);

        let train_report = Self::stations_visit(dest).await?;
        {
            let station_errors = train_report.station_errors();
            let mut station_errors = station_errors.write().await;
            dest.stations_mut().for_each(|mut station| {
                if let Some(error) = station.progress.error.take() {
                    station_errors.insert(station.rt_id, error);
                }
            });
        }

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
                let progress_bar = station_progress.borrow().progress_bar.clone();
                let progress_bar_for_tick = station_progress.borrow().progress_bar.clone();
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
            if !station.progress.progress_bar.is_finished() {
                station.progress.progress_bar.finish_at_current_pos();
            }
        });

        progress_fut
            .await
            .map_err(Error::MultiProgressTaskJoin)?
            .map_err(Error::MultiProgressJoin)?;

        Ok(())
    }

    async fn stations_visit(dest: &mut Destination<E>) -> Result<TrainReport<E>, Error<E>> {
        // Set `NotReady` stations to `Queued` if they have no dependencies.
        VisitStatusUpdater::update(dest);

        IntegrityStrat::iter(dest, TrainReport::new(), |dest, mut station, report| {
            Box::pin(async move {
                // Because this is in an async block, concurrent tasks may access this station's
                // `visit_status` while the `visit()` is `await`ed.
                station.progress.visit_status = VisitStatus::InProgress;

                match Driver::ensure(&mut station, report).await {
                    Ok(EnsureOutcomeOk::Changed) => {
                        station.progress.visit_status = VisitStatus::VisitSuccess
                    }
                    Ok(EnsureOutcomeOk::Unchanged) => {
                        station.progress.visit_status = VisitStatus::VisitUnnecessary
                    }
                    Err(EnsureOutcomeErr::CheckFail(e)) => {
                        station.progress.error = Some(e);
                        station.progress.visit_status = VisitStatus::CheckFail;
                    }
                    Err(EnsureOutcomeErr::VisitFail(e)) => {
                        station.progress.error = Some(e);
                        station.progress.visit_status = VisitStatus::VisitFail;
                    }
                }

                VisitStatusUpdater::update_children(dest, station.rt_id);

                report
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use tokio::runtime;

    use super::Train;
    use crate::{
        cfg_model::{
            StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec,
            StationSpecFns, VisitStatus,
        },
        rt_model::{indexmap::IndexMap, Destination, StationProgresses, StationRtId, StationSpecs},
    };

    #[test]
    fn reaches_empty_dest() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut dest = Destination::<()>::default();

        let train_report = rt.block_on(Train::reach(&mut dest))?;

        let station_errors = train_report.station_errors();
        assert!(
            station_errors
                .try_read()
                .expect("Expected to read station_errors.")
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn visits_all_stations_to_destination() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut dest = {
            let mut station_specs = StationSpecs::new();
            let mut station_progresses = StationProgresses::new();
            add_station(
                &mut station_specs,
                &mut station_progresses,
                "a",
                VisitStatus::Queued,
                Ok(()),
            )?;
            add_station(
                &mut station_specs,
                &mut station_progresses,
                "b",
                VisitStatus::Queued,
                Ok(()),
            )?;
            Destination::new(station_specs, station_progresses)
        };
        let train_report = rt.block_on(Train::reach(&mut dest))?;

        let station_errors = train_report.station_errors();
        assert!(
            station_errors
                .try_read()
                .expect("Expected to read station_errors.")
                .is_empty()
        );
        assert!(
            dest.station_progresses()
                .values()
                .all(|station_progress| station_progress.borrow().visit_status
                    == VisitStatus::VisitSuccess)
        );

        Ok(())
    }

    #[test]
    fn records_successful_and_failed_visits() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let (mut dest, station_a, station_b) = {
            let mut station_specs = StationSpecs::new();
            let mut station_progresses = StationProgresses::new();
            let station_a = add_station(
                &mut station_specs,
                &mut station_progresses,
                "a",
                VisitStatus::Queued,
                Ok(()),
            )?;
            let station_b = add_station(
                &mut station_specs,
                &mut station_progresses,
                "b",
                VisitStatus::Queued,
                Err(()),
            )?;
            let dest = Destination::new(station_specs, station_progresses);

            (dest, station_a, station_b)
        };
        let train_report = rt.block_on(Train::reach(&mut dest))?;

        let errors_expected = {
            let mut errors = IndexMap::new();
            errors.insert(station_b, ());
            errors
        };

        let station_errors = train_report.station_errors();
        assert_eq!(
            &errors_expected,
            &*station_errors
                .try_read()
                .expect("Expected to read station_errors.")
        );
        assert_eq!(
            VisitStatus::VisitSuccess,
            dest.station_progresses()[&station_a].borrow().visit_status
        );
        assert_eq!(
            VisitStatus::VisitFail,
            dest.station_progresses()[&station_b].borrow().visit_status
        );

        Ok(())
    }

    fn add_station(
        station_specs: &mut StationSpecs<()>,
        station_progresses: &mut StationProgresses<()>,
        station_id: &'static str,
        visit_status: VisitStatus,
        visit_result: Result<(), ()>,
    ) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
        let name = String::from(station_id);
        let station_id = StationId::new(station_id)?;
        let station_spec_fns = {
            let visit_fn = if visit_result.is_ok() {
                StationFn::new(|_, _| Box::pin(async move { Result::<(), ()>::Ok(()) }))
            } else {
                StationFn::new(|_, _| Box::pin(async move { Result::<(), ()>::Err(()) }))
            };
            StationSpecFns::new(visit_fn)
        };
        let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
        let station_progress = StationProgress::new(&station_spec, visit_status);
        let station_rt_id = station_specs.add_node(station_spec);

        station_progresses.insert(station_rt_id, station_progress);

        Ok(station_rt_id)
    }
}

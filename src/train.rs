use indicatif::MultiProgress;
use resman::Resources;
use tokio::sync::RwLock;

use crate::{
    rt_logic::{strategy::IntegrityStrat, Driver, VisitStatusUpdater},
    rt_model::{
        error::StationSpecError, Destination, EnsureOutcomeErr, EnsureOutcomeOk, Files, RwFiles,
        TrainReport, VisitStatus,
    },
};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub async fn reach<E>(dest: &mut Destination<E>) -> TrainReport<E>
    where
        E: From<StationSpecError>,
    {
        let multi_progress = MultiProgress::new();
        // Iterate using `stations` because these are sorted.
        dest.stations()
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

        let multi_progress_fut =
            tokio::task::spawn_blocking(move || multi_progress.join().unwrap());

        let mut train_report = TrainReport::new();
        let mut resources = Resources::default();
        resources.insert(RwFiles::new(RwLock::new(Files::new())));

        // Set `NotReady` stations to `Queued` if they have no dependencies.
        VisitStatusUpdater::update(dest);

        let resources = IntegrityStrat::iter(dest, resources, |dest, mut station, resources| {
            Box::pin(async move {
                // Because this is in an async block, concurrent tasks may access this station's
                // `visit_status` while the `visit()` is `await`ed.
                station.progress.visit_status = VisitStatus::InProgress;

                match Driver::ensure(&mut station, resources).await {
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

                resources
            })
        })
        .await;
        train_report.resources = resources;

        dest.stations()
            .iter()
            .filter_map(|station_spec| {
                dest.station_id_to_rt_id()
                    .get(station_spec.id())
                    .and_then(|station_rt_id| {
                        dest.station_progresses()
                            .get(station_rt_id)
                            .map(|station_progress| (*station_rt_id, station_progress))
                    })
                    .and_then(|(station_rt_id, station_progress)| {
                        station_progress
                            .borrow_mut()
                            .error
                            .take()
                            .map(|error| (station_rt_id, error))
                    })
            })
            .for_each(|(station_rt_id, error)| {
                train_report.errors.insert(station_rt_id, error);
            });

        // We need to finish / abandon all progress bars, otherwise the
        // `MultiProgress` will never finish.
        let dest = &dest;
        dest.stations().iter().for_each(move |station_spec| {
            let station_rt_id = dest.station_id_to_rt_id().get(station_spec.id()).copied();
            if let Some(station_rt_id) = station_rt_id {
                let station_progress = dest.station_progresses().try_borrow_mut(&station_rt_id);
                if let Some(station_progress) = station_progress {
                    if !station_progress.progress_bar.is_finished() {
                        station_progress.progress_bar.finish_at_current_pos();
                    }
                }
            }
        });

        // Note: This will hang if a progress bar is started but not completed.
        multi_progress_fut.await.unwrap();

        train_report
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use tokio::runtime;

    use super::Train;
    use crate::{
        cfg_model::{
            StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec, StationSpecFns,
        },
        rt_model::{Destination, StationProgresses, StationRtId, Stations, VisitStatus},
    };

    #[test]
    fn reaches_empty_dest() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut dest = Destination::<()>::default();

        let train_report = rt.block_on(Train::reach(&mut dest));

        assert!(train_report.errors.is_empty());
        Ok(())
    }

    #[test]
    fn visits_all_stations_to_destination() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut dest = {
            let mut stations = Stations::new();
            let mut station_progresses = StationProgresses::new();
            add_station(
                &mut stations,
                &mut station_progresses,
                "a",
                VisitStatus::Queued,
                Ok(()),
            )?;
            add_station(
                &mut stations,
                &mut station_progresses,
                "b",
                VisitStatus::Queued,
                Ok(()),
            )?;
            Destination::new(stations, station_progresses)
        };
        let train_report = rt.block_on(Train::reach(&mut dest));

        assert!(train_report.errors.is_empty());
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
            let mut stations = Stations::new();
            let mut station_progresses = StationProgresses::new();
            let station_a = add_station(
                &mut stations,
                &mut station_progresses,
                "a",
                VisitStatus::Queued,
                Ok(()),
            )?;
            let station_b = add_station(
                &mut stations,
                &mut station_progresses,
                "b",
                VisitStatus::Queued,
                Err(()),
            )?;
            let dest = Destination::new(stations, station_progresses);

            (dest, station_a, station_b)
        };
        let train_report = rt.block_on(Train::reach(&mut dest));

        let errors_expected = {
            let mut errors = IndexMap::new();
            errors.insert(station_b, ());
            errors
        };
        assert_eq!(&errors_expected, &train_report.errors);
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
        stations: &mut Stations<()>,
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
        let station_rt_id = stations.add_node(station_spec);

        station_progresses.insert(station_rt_id, station_progress);

        Ok(station_rt_id)
    }
}

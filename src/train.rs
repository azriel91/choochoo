use daggy::{
    petgraph::{graph::DefaultIx, visit::IntoNodeIdentifiers},
    NodeIndex,
};
use indicatif::MultiProgress;
use resman::Resources;

use crate::{
    rt_logic::{strategy::IntegrityStrat, Driver},
    rt_model::{
        error::StationSpecError, Destination, EnsureOutcome, Files, TrainReport, VisitStatus,
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
        dest.stations.iter().for_each(|station| {
            let progress_bar = station.progress_bar.clone();
            let progress_bar_for_tick = station.progress_bar.clone();
            multi_progress.add(progress_bar);

            // Needed to render all progress bars.
            progress_bar_for_tick.tick();
        });

        let multi_progress_fut =
            tokio::task::spawn_blocking(move || multi_progress.join().unwrap());

        let mut train_report = TrainReport::new();
        let mut resources = Resources::default();
        resources.insert(Files::new());
        let resources = IntegrityStrat::iter(dest, resources, |resources, station| {
            Box::pin(async move {
                // Because this is in an async block, concurrent tasks may access this station's
                // `visit_status` while the `visit()` is `await`ed.
                station.visit_status = VisitStatus::InProgress;

                match Driver::ensure(resources, station).await {
                    Ok(EnsureOutcome::Changed) => station.visit_status = VisitStatus::VisitSuccess,
                    Ok(EnsureOutcome::Unchanged) => {
                        station.visit_status = VisitStatus::VisitUnnecessary
                    }
                    Err(_e) => {
                        station.visit_status = VisitStatus::VisitFail;
                    }
                }
                resources
            })
        })
        .await;
        train_report.resources = resources;

        let node_ids = dest
            .stations
            .node_identifiers()
            .collect::<Vec<NodeIndex<DefaultIx>>>();
        let mut frozen = dest.stations.frozen();

        node_ids
            .into_iter()
            .filter_map(|node_id| {
                let station = &mut frozen[node_id];
                station.error.take().map(|error| (node_id, error))
            })
            .for_each(|(node_id, error)| {
                train_report.errors.insert(node_id, error);
            });

        multi_progress_fut.await.unwrap();

        train_report
    }
}

#[cfg(test)]
mod tests {
    use daggy::{petgraph::graph::DefaultIx, NodeIndex};
    use indexmap::IndexMap;
    use tokio::runtime;

    use super::Train;
    use crate::{
        cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns},
        rt_model::{Destination, Station, Stations, VisitStatus},
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
            add_station(&mut stations, "a", VisitStatus::Queued, Ok(()))?;
            add_station(&mut stations, "b", VisitStatus::Queued, Ok(()))?;
            Destination { stations }
        };
        let train_report = rt.block_on(Train::reach(&mut dest));

        assert!(train_report.errors.is_empty());
        assert!(
            dest.stations
                .iter()
                .all(|station| station.visit_status == VisitStatus::VisitSuccess)
        );

        Ok(())
    }

    #[test]
    fn records_successful_and_failed_visits() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let (mut dest, station_a, station_b) = {
            let mut stations = Stations::new();
            let station_a = add_station(&mut stations, "a", VisitStatus::Queued, Ok(()))?;
            let station_b = add_station(&mut stations, "b", VisitStatus::Queued, Err(()))?;
            let dest = Destination { stations };

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
            dest.stations[station_a].visit_status
        );
        assert_eq!(
            VisitStatus::VisitFail,
            dest.stations[station_b].visit_status
        );

        Ok(())
    }

    fn add_station(
        stations: &mut Stations<()>,
        station_id: &'static str,
        visit_status: VisitStatus,
        visit_result: Result<(), ()>,
    ) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
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
        let station = Station::new(station_spec, visit_status);
        Ok(stations.add_node(station))
    }
}

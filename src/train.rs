use crate::{
    rt_logic::IntegrityStrat,
    rt_model::{Destination, TrainReport, VisitStatus},
};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub async fn reach<'files, E>(dest: &mut Destination<E>) -> TrainReport<'files, E>
    where
        E: Send + Sync,
    {
        let train_report = TrainReport::new();
        IntegrityStrat::iter(dest, train_report, |mut train_report, node_id, station| {
            Box::pin(async move {
                // Because this is in an async block, concurrent tasks may access this station's
                // `visit_status` while the `visit()` is `await`ed.
                station.visit_status = VisitStatus::InProgress;

                if let Err(e) = station.visit().await {
                    station.visit_status = VisitStatus::VisitFail;
                    train_report.errors.insert(node_id, e);
                } else {
                    station.visit_status = VisitStatus::VisitSuccess;
                }
                train_report
            })
        })
        .await
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
                StationFn::new(|_station| Box::pin(async move { Result::<(), ()>::Ok(()) }))
            } else {
                StationFn::new(|_station| Box::pin(async move { Result::<(), ()>::Err(()) }))
            };
            StationSpecFns { visit_fn }
        };
        let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
        let station = Station::new(station_spec, visit_status);
        Ok(stations.add_node(station))
    }
}

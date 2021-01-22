use crate::{
    rt_logic::IntegrityStrat,
    rt_model::{Destination, TrainReport, VisitStatus},
};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub async fn reach<'dest, E>(dest: &'dest mut Destination<E>) -> TrainReport
    where
        E: 'dest,
    {
        let train_report = TrainReport::new();
        IntegrityStrat::iter(
            dest,
            train_report,
            |mut train_report, station_id, station| {
                Box::pin(async move {
                    // Because this is in an async block, concurrent tasks may access this station's
                    // `visit_status` while the `visit()` is `await`ed.
                    station.visit_status = VisitStatus::InProgress;

                    if let Err(_e) = station.visit().await {
                        station.visit_status = VisitStatus::VisitFail;
                        train_report.stations_failed.push(station_id);
                    } else {
                        station.visit_status = VisitStatus::VisitSuccess;
                        train_report.stations_successful.push(station_id);
                    }
                    train_report
                })
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use tokio::runtime;

    use super::Train;
    use crate::{
        cfg_model::{StationSpec, VisitFn},
        rt_model::{Destination, Station, Stations, VisitStatus},
    };

    #[test]
    fn reaches_empty_dest() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut dest = Destination::<()>::default();

        let train_report = rt.block_on(Train::reach(&mut dest));

        assert!(train_report.stations_successful.is_empty());
        Ok(())
    }

    #[test]
    fn visits_all_stations_to_destination() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, VisitStatus::Queued, Ok(()));
            add_station(&mut stations, VisitStatus::Queued, Ok(()));
            Destination { stations }
        };
        let train_report = rt.block_on(Train::reach(&mut dest));

        assert_eq!(2, train_report.stations_successful.len());
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
        let mut dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, VisitStatus::Queued, Ok(()));
            add_station(&mut stations, VisitStatus::Queued, Err(()));
            Destination { stations }
        };
        let train_report = rt.block_on(Train::reach(&mut dest));

        let frozen = dest.stations.frozen();
        assert_eq!(1, train_report.stations_successful.len());
        assert_eq!(
            VisitStatus::VisitSuccess,
            frozen[train_report.stations_successful[0]].visit_status
        );
        assert_eq!(1, train_report.stations_failed.len());
        assert_eq!(
            VisitStatus::VisitFail,
            frozen[train_report.stations_failed[0]].visit_status
        );

        Ok(())
    }

    fn add_station(
        stations: &mut Stations<()>,
        visit_status: VisitStatus,
        visit_result: Result<(), ()>,
    ) {
        let visit_fn = if visit_result.is_ok() {
            VisitFn::new(|_station| Box::pin(async move { Result::<(), ()>::Ok(()) }))
        } else {
            VisitFn::new(|_station| Box::pin(async move { Result::<(), ()>::Err(()) }))
        };
        let station_spec = StationSpec::new(visit_fn);
        let station = Station::new(station_spec, visit_status);
        stations.add_node(station);
    }
}

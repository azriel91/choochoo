use futures::stream::{self, StreamExt};

use crate::rt_model::{Destination, TrainReport};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub async fn reach<'dest, E>(dest: &'dest mut Destination<E>) -> TrainReport<'dest, E>
    where
        E: 'dest,
    {
        stream::iter(dest.stations.iter_mut())
            .fold(TrainReport::new(), |mut train_report, station| async move {
                if let Err(_e) = station.visit().await {
                    train_report.stations_failed.push(station);
                } else {
                    train_report.stations_successful.push(station);
                }
                train_report
            })
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
            let _a = {
                let station_spec = StationSpec::new(VisitFn(|station| {
                    Box::pin(async move {
                        station.visit_status = VisitStatus::VisitSuccess;
                        Result::<(), ()>::Ok(())
                    })
                }));
                let station = Station::new(station_spec, VisitStatus::Queued);
                stations.add_node(station)
            };
            let _b = {
                let station_spec = StationSpec::new(VisitFn(|station| {
                    Box::pin(async move {
                        station.visit_status = VisitStatus::VisitSuccess;
                        Result::<(), ()>::Ok(())
                    })
                }));
                let station = Station::new(station_spec, VisitStatus::Queued);
                stations.add_node(station)
            };
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
            let _a = {
                let station_spec = StationSpec::new(VisitFn(|station| {
                    Box::pin(async move {
                        station.visit_status = VisitStatus::VisitSuccess;
                        Result::<(), ()>::Ok(())
                    })
                }));
                let station = Station::new(station_spec, VisitStatus::Queued);
                stations.add_node(station)
            };
            let _b = {
                let station_spec = StationSpec::new(VisitFn(|station| {
                    Box::pin(async move {
                        station.visit_status = VisitStatus::VisitSuccess;
                        Result::<(), ()>::Err(())
                    })
                }));
                let station = Station::new(station_spec, VisitStatus::Queued);
                stations.add_node(station)
            };
            Destination { stations }
        };
        let train_report = rt.block_on(Train::reach(&mut dest));

        assert_eq!(1, train_report.stations_successful.len());
        assert_eq!(1, train_report.stations_failed.len());

        Ok(())
    }
}

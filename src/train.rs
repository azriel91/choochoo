use futures::stream::{self, StreamExt};

use crate::rt_model::{Destination, Station};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub async fn reach<D>(dest: &mut D)
    where
        D: Destination,
    {
        stream::iter(dest.stations_mut().iter_mut())
            .for_each(Station::visit)
            .await;
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
        let mut dest = TestDest::default();

        rt.block_on(Train::reach(&mut dest));

        Ok(())
    }

    #[test]
    fn visits_stations_to_destination() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut dest = {
            let mut stations = Stations::new();
            let _a = {
                let station_spec = StationSpec::new(VisitFn(|station| {
                    Box::pin(async move { station.visit_status = VisitStatus::VisitSuccess })
                }));
                let station = Station::new(station_spec, VisitStatus::Queued);
                stations.add_node(station)
            };
            let _b = {
                let station_spec = StationSpec::new(VisitFn(|station| {
                    Box::pin(async move { station.visit_status = VisitStatus::VisitSuccess })
                }));
                let station = Station::new(station_spec, VisitStatus::Queued);
                stations.add_node(station)
            };
            TestDest { stations }
        };
        rt.block_on(Train::reach(&mut dest));

        assert!(
            dest.stations()
                .iter()
                .all(|station| station.visit_status == VisitStatus::VisitSuccess)
        );

        Ok(())
    }

    #[derive(Debug, Default)]
    struct TestDest {
        stations: Stations,
    }

    impl Destination for TestDest {
        fn stations(&self) -> &Stations {
            &self.stations
        }

        fn stations_mut(&mut self) -> &mut Stations {
            &mut self.stations
        }
    }
}

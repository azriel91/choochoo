use futures::stream::{self, StreamExt};

use crate::{Destination, Station};

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
    use crate::{Destination, Station, Stations, VisitFn, VisitStatus};

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
            let _a = stations.add_node(Station::new(
                VisitStatus::Queued,
                VisitFn(|station| {
                    Box::pin(async move { station.visit_status = VisitStatus::VisitSuccess })
                }),
            ));
            let _b = stations.add_node(Station::new(
                VisitStatus::Queued,
                VisitFn(|station| {
                    Box::pin(async move { station.visit_status = VisitStatus::VisitSuccess })
                }),
            ));
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

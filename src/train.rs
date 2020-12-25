use crate::{Destination, Station};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub fn reach<D>(dest: &mut D)
    where
        D: Destination,
    {
        dest.stations_mut().iter_mut().for_each(Station::visit);
    }
}

#[cfg(test)]
mod tests {
    use super::Train;
    use crate::{Destination, Station, Stations, VisitStatus};

    #[test]
    fn reaches_empty_dest() {
        let mut dest = TestDest::default();
        Train::reach(&mut dest);
    }

    #[test]
    fn visits_stations_to_destination() {
        let mut dest = {
            let mut stations = Stations::new();
            let _a = stations.add_node(Station::new(VisitStatus::Queued));
            let _b = stations.add_node(Station::new(VisitStatus::Queued));
            TestDest { stations }
        };
        Train::reach(&mut dest);

        assert!(
            dest.stations()
                .iter()
                .all(|station| station.visit_status() == VisitStatus::Visited)
        )
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

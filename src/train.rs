use crate::{Destination, Station};

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub fn reach<D>(mut dest: D)
    where
        D: Destination,
    {
        if dest.is_reached() {
            // TODO: Report
        } else {
            dest.stations().iter_mut().for_each(Station::visit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Train;
    use crate::{Destination, Station, Stations, VisitStatus};

    #[test]
    fn reaches_empty_dest() {
        let dest = EmptyDest::default();
        Train::reach(dest);
    }

    #[test]
    fn visits_stations_to_destination() {
        let dest = {
            let mut stations = Stations::new();
            let _a = stations.add_node(Station::new(VisitStatus::Queued));
            let _b = stations.add_node(Station::new(VisitStatus::Queued));
            TestDest { stations }
        };
        Train::reach(dest);
    }

    #[derive(Debug, Default)]
    struct EmptyDest {
        stations: Stations,
    }

    impl Destination for EmptyDest {
        fn is_reached(&self) -> bool {
            true
        }

        fn stations(&mut self) -> &mut Stations {
            &mut self.stations
        }
    }

    #[derive(Debug)]
    struct TestDest {
        stations: Stations,
    }

    impl Destination for TestDest {
        fn is_reached(&self) -> bool {
            self.stations
                .iter()
                .all(|station| station.visit_status() == VisitStatus::Visited)
        }

        fn stations(&mut self) -> &mut Stations {
            &mut self.stations
        }
    }
}

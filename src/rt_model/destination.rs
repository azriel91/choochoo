use std::iter::{Filter, Peekable};

use daggy::{petgraph::graph::DefaultIx, NodeWeightsMut};

use crate::rt_model::{Station, Stations, VisitStatus};

/// Iterator over [`Station`]s that have [`VisitStatus::Queued`].
pub type StationsQueuedIter<'dest, E> =
    Peekable<Filter<NodeWeightsMut<'dest, Station<E>, DefaultIx>, FnFilterQueued<E>>>;

type FnFilterQueued<E> = for<'f, 's> fn(&'f &'s mut Station<E>) -> bool;

/// Specification of a desired state.
#[derive(Clone, Debug, Default)]
pub struct Destination<E> {
    /// The stations along the way to the destination.
    pub stations: Stations<E>,
}

impl<E> Destination<E> {
    /// Returns an iterator over queued `Station`s, or `None` if none are
    /// queued.
    ///
    /// This does not include `Station`s that have a visit in progress.
    pub fn stations_queued(&mut self) -> Option<StationsQueuedIter<'_, E>> {
        // Without this, we get the error:
        //
        // ```
        // error[E0308]: mismatched types
        //   --> src/rt_model/destination.rs:35:18
        //    |
        // 35 |             Some(stations_queued)
        //    |                  ^^^^^^^^^^^^^^^ expected fn pointer, found fn item
        //    |
        // ```
        //
        // https://users.rust-lang.org/t/difference-between-fn-pointer-and-fn-item/32642
        let is_station_queued: FnFilterQueued<E> = Self::is_station_queued;

        let mut stations_queued = self
            .stations
            .iter_mut()
            .filter(is_station_queued)
            .peekable();

        if stations_queued.peek().is_some() {
            Some(stations_queued)
        } else {
            None
        }
    }

    fn is_station_queued(station: &&mut Station<E>) -> bool {
        station.visit_status == VisitStatus::Queued
    }
}

#[cfg(test)]
mod tests {
    use super::Destination;
    use crate::{
        cfg_model::{StationId, StationSpec, VisitFn},
        rt_model::{Station, Stations, VisitStatus},
    };

    #[test]
    fn stations_queued_returns_none_when_no_stations_queued() {
        let mut dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, "a", VisitStatus::VisitSuccess);
            add_station(&mut stations, "b", VisitStatus::NotReady);
            Destination { stations }
        };

        assert!(
            dest.stations_queued().is_none(),
            "Expected `stations_queued()` to be `None`."
        );
    }

    #[test]
    fn stations_queued_returns_iter_when_stations_queued_exists() {
        let mut dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, "a", VisitStatus::Queued);
            add_station(&mut stations, "b", VisitStatus::Queued);
            add_station(&mut stations, "c", VisitStatus::NotReady);
            Destination { stations }
        };

        if let Some(mut stations_queued) = dest.stations_queued() {
            assert!(stations_queued.next().is_some());
            assert!(stations_queued.next().is_some());
            assert!(stations_queued.next().is_none());
        } else {
            panic!("Expected stations_queued to be `Some(..)`");
        }
    }

    fn add_station(
        stations: &mut Stations<()>,
        station_id: &'static str,
        visit_status: VisitStatus,
    ) {
        let name = String::from(station_id);
        let station_id = StationId::new(station_id).unwrap();
        let visit_fn = VisitFn::new(|_station| Box::pin(async move { Result::<(), ()>::Ok(()) }));
        let station_spec = StationSpec::new(station_id, name, String::from(""), visit_fn);
        let station = Station::new(station_spec, visit_status);
        stations.add_node(station);
    }
}

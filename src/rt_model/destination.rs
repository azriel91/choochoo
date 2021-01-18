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

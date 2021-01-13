use std::iter::Filter;

use daggy::{petgraph::graph::DefaultIx, NodeWeightsMut};

use crate::rt_model::{Station, Stations, VisitStatus};

/// Specification of a desired state.
#[derive(Clone, Debug, Default)]
pub struct Destination<E> {
    /// The stations along the way to the destination.
    pub stations: Stations<E>,
}

impl<E> Destination<E> {
    /// Returns an iterator over the `Station`s that are ready to be visited.
    ///
    /// This does not include `Station`s that have a visit in progress.
    pub fn stations_queued(
        &mut self,
    ) -> Filter<NodeWeightsMut<Station<E>, DefaultIx>, fn(&&mut Station<E>) -> bool> {
        self.stations
            .iter_mut()
            .filter(|station| station.visit_status == VisitStatus::Queued)
    }
}

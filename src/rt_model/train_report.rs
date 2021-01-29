use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use indexmap::IndexMap;

/// Record of what happened during a train's drive.
#[derive(Debug, PartialEq)]
pub struct TrainReport<E> {
    /// Stations that were visited but failed to work.
    pub errors: IndexMap<NodeIndex<DefaultIx>, E>,
}

impl<E> Default for TrainReport<E> {
    fn default() -> Self {
        Self {
            errors: Default::default(),
        }
    }
}

impl<E> TrainReport<E> {
    /// Returns a new TrainReport.
    pub fn new() -> Self {
        Self::default()
    }
}

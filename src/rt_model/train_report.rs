use std::fmt;

use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use indexmap::IndexMap;
use resman::Resources;

use crate::rt_model::Files;

/// Record of what happened during a train's drive.
pub struct TrainReport<E> {
    /// Stations that were visited but failed to work.
    pub errors: IndexMap<NodeIndex<DefaultIx>, E>,
    /// Resources used during execution.
    pub resources: Resources,
}

impl<E> TrainReport<E> {
    /// Returns a new TrainReport.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<E> fmt::Debug for TrainReport<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TrainReport")
            .field("errors", &self.errors)
            .field("resources", &"resman::Resources { .. }")
            .finish()
    }
}

impl<E> Default for TrainReport<E> {
    fn default() -> Self {
        let mut resources = Resources::default();
        resources.insert(Files::new());

        Self {
            errors: Default::default(),
            resources,
        }
    }
}

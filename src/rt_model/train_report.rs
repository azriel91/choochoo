use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use indexmap::IndexMap;
use resman::Resources;
use tokio::sync::RwLock;

use crate::rt_model::{Files, RwFiles};

/// Record of what happened during a train's drive.
#[derive(Debug)]
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

impl<E> Default for TrainReport<E> {
    fn default() -> Self {
        let mut resources = Resources::default();
        resources.insert(RwFiles::new(RwLock::new(Files::new())));

        Self {
            errors: Default::default(),
            resources,
        }
    }
}

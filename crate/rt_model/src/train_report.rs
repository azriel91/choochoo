use choochoo_cfg_model::resman::Resources;
use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use indexmap::IndexMap;

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
    pub fn new(resources: Resources) -> Self {
        Self {
            errors: Default::default(),
            resources,
        }
    }
}

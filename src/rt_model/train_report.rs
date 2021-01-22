use daggy::{petgraph::graph::DefaultIx, NodeIndex};

/// Record of what happened during a train's drive.
#[derive(Debug, PartialEq)]
pub struct TrainReport {
    /// Stations successfully visited.
    pub stations_successful: Vec<NodeIndex<DefaultIx>>,
    /// Stations that were visited but failed to work.
    pub stations_failed: Vec<NodeIndex<DefaultIx>>,
}

impl Default for TrainReport {
    fn default() -> Self {
        Self {
            stations_successful: Default::default(),
            stations_failed: Default::default(),
        }
    }
}

impl TrainReport {
    /// Returns a new TrainReport.
    pub fn new() -> Self {
        Self::default()
    }
}

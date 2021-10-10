use daggy::{petgraph::graph::DefaultIx, NodeIndex};

/// Runtime identifier for a station.
///
/// This is a cheaper identifier than [`StationId`] to copy around.
///
/// [`StationId`]: choochoo_cfg_model::StationId
pub type StationRtId = NodeIndex<DefaultIx>;

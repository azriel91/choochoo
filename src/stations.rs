use daggy::Dag;

use crate::{Station, Workload};

/// Directed acyclic graph of [`Station`]s.
pub type Stations = Dag<Station, Workload>;

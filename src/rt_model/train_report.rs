use std::borrow::Cow;

use codespan::Files;
use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use indexmap::IndexMap;

/// Record of what happened during a train's drive.
#[derive(Debug)]
pub struct TrainReport<'files, E> {
    /// Stations that were visited but failed to work.
    pub errors: IndexMap<NodeIndex<DefaultIx>, E>,
    /// Content of files / data referenced in errors.
    pub files: Files<Cow<'files, str>>,
}

impl<'files, E> Default for TrainReport<'files, E> {
    fn default() -> Self {
        Self {
            errors: Default::default(),
            files: Default::default(),
        }
    }
}

impl<'files, E> TrainReport<'files, E> {
    /// Returns a new TrainReport.
    pub fn new() -> Self {
        Self::default()
    }
}

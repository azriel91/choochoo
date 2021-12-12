use std::ops::{Deref, DerefMut};

use fn_graph::FnGraph;

use crate::StationSpec;

/// Directed acyclic graph of [`StationSpec`]s.
#[derive(Clone, Debug)]
pub struct StationSpecs<E>(pub FnGraph<StationSpec<E>>);

impl<E> StationSpecs<E> {
    /// Returns an empty graph of [`StationSpec`]s.
    pub fn new(station_specs: FnGraph<StationSpec<E>>) -> Self {
        Self(station_specs)
    }
}

impl<E> Default for StationSpecs<E> {
    fn default() -> Self {
        Self(FnGraph::default())
    }
}

impl<E> Deref for StationSpecs<E> {
    type Target = FnGraph<StationSpec<E>>;

    #[cfg(not(tarpaulin_include))]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for StationSpecs<E> {
    #[cfg(not(tarpaulin_include))]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

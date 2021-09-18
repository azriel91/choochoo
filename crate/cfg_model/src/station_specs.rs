use std::ops::{Deref, DerefMut};

use daggy::{
    petgraph::graph::{DefaultIx, Frozen, NodeReferences},
    Dag, NodeWeightsMut,
};

use crate::{StationSpec, Workload};

/// Frozen station spec graph.
pub type StationsFrozen<'s, E> = Frozen<'s, Dag<StationSpec<E>, Workload>>;

/// Directed acyclic graph of [`StationSpec`]s.
#[derive(Clone, Debug, Default)]
pub struct StationSpecs<E>(pub Dag<StationSpec<E>, Workload>);

impl<E> StationSpecs<E> {
    /// Returns an empty graph of [`StationSpec`]s.
    pub fn new() -> Self {
        Self(Dag::new())
    }

    /// Returns a frozen stations graph.
    pub fn frozen(&mut self) -> StationsFrozen<'_, E> {
        Frozen::new(&mut self.0)
    }

    /// Returns an iterator over references of all [`StationSpec`]s.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &StationSpec<E>> + ExactSizeIterator + DoubleEndedIterator {
        use daggy::petgraph::visit::IntoNodeReferences;
        self.0
            .node_references()
            .map(|(_, station_spec)| station_spec)
    }

    /// Returns an iterator over mutable references of all [`StationSpec`]s.
    pub fn iter_mut(&mut self) -> NodeWeightsMut<StationSpec<E>, DefaultIx> {
        self.0.node_weights_mut()
    }

    /// Returns an iterator over references of all [`StationSpec`]s.
    ///
    /// Each iteration returns a `(NodeIndex<Ix>, &'a N)`.
    pub fn iter_with_indices(&self) -> NodeReferences<StationSpec<E>> {
        use daggy::petgraph::visit::IntoNodeReferences;
        self.0.node_references()
    }
}

impl<E> Deref for StationSpecs<E> {
    type Target = Dag<StationSpec<E>, Workload>;

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

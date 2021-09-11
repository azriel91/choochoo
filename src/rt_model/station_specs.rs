use std::ops::{Deref, DerefMut};

use daggy::{
    petgraph::graph::{DefaultIx, Frozen, NodeReferences},
    Dag, NodeWeightsMut,
};

use crate::cfg_model::{StationSpec, Workload};

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

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use crate::rt_model::StationRtId;
    use daggy::NodeIndex;

    use super::StationSpecs;
    use crate::{
        cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns},
        rt_model::VisitStatus,
    };

    #[test]
    fn iter_with_indices_returns_iterator_with_all_stations()
    -> Result<(), StationIdInvalidFmt<'static>> {
        let mut station_specs = StationSpecs::new();
        let a = add_station(&mut station_specs, "a")?;
        let b = add_station(&mut station_specs, "b")?;

        let indicies = station_specs
            .iter_with_indices()
            .map(|(node_index, _)| node_index)
            .collect::<Vec<NodeIndex>>();

        assert_eq!(vec![a, b], indicies);
        Ok(())
    }

    #[test]
    fn deref() {
        let station_specs = StationSpecs::<()>::new();
        assert!(std::ptr::eq(Deref::deref(&station_specs), &station_specs.0));
    }

    #[test]
    fn deref_mut() {
        let mut station_specs = StationSpecs::<()>::new();
        assert!(std::ptr::eq(
            DerefMut::deref_mut(&mut station_specs),
            &mut station_specs.0
        ));
    }

    fn add_station(
        station_specs: &mut StationSpecs<()>,
        station_id: &'static str,
    ) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
        let name = String::from(station_id);
        let station_id = StationId::new(station_id)?;
        let station_spec_fns = {
            let visit_fn = StationFn::new(|station_progress, _| {
                Box::pin(async move {
                    station_progress.visit_status = VisitStatus::VisitSuccess;
                    Result::<(), ()>::Ok(())
                })
            });
            StationSpecFns::new(visit_fn)
        };
        let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
        Ok(station_specs.add_node(station_spec))
    }
}

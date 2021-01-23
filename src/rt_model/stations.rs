use std::ops::{Deref, DerefMut};

use daggy::{
    petgraph::graph::{DefaultIx, Frozen, NodeReferences},
    Dag, NodeWeightsMut,
};

use crate::{cfg_model::Workload, rt_model::Station};

/// Frozen stations graph.
pub type StationsFrozen<'s, E> = Frozen<'s, Dag<Station<E>, Workload>>;

/// Directed acyclic graph of [`Station`]s.
#[derive(Clone, Debug, Default)]
pub struct Stations<E>(pub Dag<Station<E>, Workload>);

impl<E> Stations<E> {
    /// Returns an empty graph of [`Station`]s.
    pub fn new() -> Self {
        Self(Dag::new())
    }

    /// Returns a frozen stations graph.
    pub fn frozen(&mut self) -> StationsFrozen<'_, E> {
        Frozen::new(&mut self.0)
    }

    /// Returns an iterator over references of all [`Station`]s.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &Station<E>> + ExactSizeIterator + DoubleEndedIterator {
        use daggy::petgraph::visit::IntoNodeReferences;
        self.0.node_references().map(|(_, station)| station)
    }

    /// Returns an iterator over mutable references of all [`Station`]s.
    pub fn iter_mut(&mut self) -> NodeWeightsMut<Station<E>, DefaultIx> {
        self.0.node_weights_mut()
    }

    /// Returns an iterator over references of all [`Station`]s.
    pub fn iter_with_indices(&self) -> NodeReferences<Station<E>> {
        use daggy::petgraph::visit::IntoNodeReferences;
        self.0.node_references()
    }
}

impl<E> Deref for Stations<E> {
    type Target = Dag<Station<E>, Workload>;

    #[cfg(not(tarpaulin_include))]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for Stations<E> {
    #[cfg(not(tarpaulin_include))]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use daggy::{petgraph::graph::DefaultIx, NodeIndex};

    use super::Stations;
    use crate::{
        cfg_model::{StationId, StationSpec, VisitFn},
        rt_model::{Station, VisitStatus},
    };

    #[test]
    fn iter_with_indices_returns_iterator_with_all_stations() {
        let mut stations = Stations::new();
        let a = add_station(&mut stations, "a");
        let b = add_station(&mut stations, "b");

        let indicies = stations
            .iter_with_indices()
            .map(|(node_index, _)| node_index)
            .collect::<Vec<NodeIndex>>();

        assert_eq!(vec![a, b], indicies);
    }

    #[test]
    fn deref() {
        let stations = Stations::<()>::new();
        assert!(std::ptr::eq(Deref::deref(&stations), &stations.0));
    }

    #[test]
    fn deref_mut() {
        let mut stations = Stations::<()>::new();
        assert!(std::ptr::eq(
            DerefMut::deref_mut(&mut stations),
            &mut stations.0
        ));
    }

    fn add_station(stations: &mut Stations<()>, station_id: &'static str) -> NodeIndex<DefaultIx> {
        let name = String::from(station_id);
        let station_id = StationId::new(station_id).unwrap();
        let visit_fn = VisitFn::new(|station| {
            Box::pin(async move {
                station.visit_status = VisitStatus::VisitSuccess;
                Result::<(), ()>::Ok(())
            })
        });
        let station_spec = StationSpec::new(station_id, name, String::from(""), visit_fn);
        let station = Station::new(station_spec, VisitStatus::Queued);
        stations.add_node(station)
    }
}

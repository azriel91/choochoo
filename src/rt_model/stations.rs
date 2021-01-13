use std::ops::{Deref, DerefMut};

use daggy::{
    petgraph::graph::{DefaultIx, NodeReferences},
    Dag, NodeWeightsMut,
};

use crate::{cfg_model::Workload, rt_model::Station};

/// Directed acyclic graph of [`Station`]s.
#[derive(Clone, Debug, Default)]
pub struct Stations<E>(pub Dag<Station<E>, Workload>);

impl<E> Stations<E> {
    /// Returns an empty graph of [`Station`]s.
    pub fn new() -> Self {
        Self(Dag::new())
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

    use daggy::NodeIndex;

    use super::Stations;
    use crate::{
        cfg_model::{StationSpec, VisitFn},
        rt_model::{Station, VisitStatus},
    };

    #[test]
    fn iter_with_indices_returns_iterator_with_all_stations() {
        let mut stations = Stations::new();
        let a = {
            let station_spec = StationSpec::new(VisitFn::new(|station| {
                Box::pin(async move {
                    station.visit_status = VisitStatus::VisitSuccess;
                    Result::<(), ()>::Ok(())
                })
            }));
            let station = Station::new(station_spec, VisitStatus::Queued);
            stations.add_node(station)
        };
        let b = {
            let station_spec = StationSpec::new(VisitFn::new(|station| {
                Box::pin(async move {
                    station.visit_status = VisitStatus::VisitSuccess;
                    Result::<(), ()>::Ok(())
                })
            }));
            let station = Station::new(station_spec, VisitStatus::Queued);
            stations.add_node(station)
        };

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
}

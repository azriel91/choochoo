use std::marker::PhantomData;

use daggy::{petgraph::graph::DefaultIx, NodeIndex, Walker};

use crate::rt_model::{Station, Stations, VisitStatus};

/// Updates the [`VisitStatus`]es for all [`Station`]s.
///
/// The new visit status is calculated based on the station's visit result
/// and its parents' [`VisitStatus`]es.
///
/// # `VisitStatus` State Machine
///
/// ## `NotReady` Stations
///
/// * If all parents are `VisitSuccess`, switch to `Queued`.
/// * If at least one parent has `VisitFailed` or `ParentFail`, switch to
///   `ParentFail`.
///
/// ## `ParentFail` Stations
///
/// No transitions.
///
/// ## `Queued` Stations
///
/// No transitions -- [`Train::reach`] sets this to `InProgress` when visiting
/// the station.
///
/// ## `InProgress` Stations
///
/// No transitions -- [`Train::reach`] sets this to `VisitSuccess` or
/// `VisitFail` depending on [`Station::visit`]'s result.
///
/// ## `VisitSuccess`
///
/// No transitions.
///
/// ## `VisitFail`
///
/// No transitions.
#[derive(Debug)]
pub struct VisitStatusUpdater<E> {
    /// Marker
    pub marker: PhantomData<E>,
}

impl<E> VisitStatusUpdater<E> {
    /// Updates the [`VisitStatus`]es for all [`Station`]s.
    ///
    /// # Parameters
    ///
    /// * `stations`: `Stations` to update the `VisitStatus` for.
    pub fn update(stations: &mut Stations<E>) {
        stations.graph().node_indices().for_each(|node_index| {
            // The `Option<&mut Station>` returned by `node_weight_mut` cannot be returned
            // outside an iterator closure, which is why we cannot use `filter_map`.
            if let Some((station, visit_status_next)) = stations
                .node_weight(node_index)
                .and_then(|station| Self::visit_status_next(stations, node_index, station))
                .and_then(|visit_status_next| {
                    stations
                        .node_weight_mut(node_index)
                        .map(|station| (station, visit_status_next))
                })
            {
                station.visit_status = visit_status_next;
            }
        });
    }

    /// Returns the [`VisitStatus`] to be transitioned to, if any.
    fn visit_status_next(
        stations: &Stations<E>,
        node_index: NodeIndex<DefaultIx>,
        station: &Station<E>,
    ) -> Option<VisitStatus> {
        match station.visit_status {
            VisitStatus::NotReady => Self::transition_not_ready(stations, node_index),
            VisitStatus::ParentFail | VisitStatus::VisitSuccess | VisitStatus::VisitFail => None,
            _ => None,
        }
    }

    fn transition_not_ready(
        stations: &Stations<E>,
        node_index: NodeIndex<DefaultIx>,
    ) -> Option<VisitStatus> {
        let parents_walker = stations.parents(node_index);
        let visit_status_next = parents_walker
            .iter(stations)
            .filter_map(|(_, parent_node_index)| stations.node_weight(parent_node_index))
            .try_fold(Some(VisitStatus::Queued), |visit_status, parent_station| {
                match parent_station.visit_status {
                    // If parent is on `VisitSuccess`, we keep going.
                    VisitStatus::VisitSuccess => {}

                    // Short circuits:

                    // If parent / ancestor has failed, indicate it in this station.
                    VisitStatus::VisitFail | VisitStatus::ParentFail => {
                        return Err(Some(VisitStatus::ParentFail));
                    }
                    // Don't change `VisitStatus` if parent is on any other `VisitStatus`.
                    _ => return Err(None),
                }

                Ok(visit_status)
            });

        match visit_status_next {
            Ok(visit_status_next) | Err(visit_status_next) => visit_status_next,
        }
    }
}

#[cfg(test)]
mod tests {
    use daggy::WouldCycle;

    use super::VisitStatusUpdater;
    use crate::{
        cfg_model::{StationSpec, VisitFn, Workload},
        rt_model::{Station, Stations, VisitStatus},
    };

    #[test]
    fn updates_not_ready_to_queued_when_no_parents_exist() {
        let mut stations = Stations::new();
        let station = station(VisitStatus::NotReady);
        let node_index = stations.add_node(station);
        let station = stations.node_weight(node_index).unwrap();

        let visit_status_next =
            VisitStatusUpdater::visit_status_next(&stations, node_index, &station);

        assert_eq!(Some(VisitStatus::Queued), visit_status_next);
    }

    #[test]
    fn updates_not_ready_to_queued_when_all_parents_visit_success()
    -> Result<(), WouldCycle<Workload>> {
        // a -> c
        //      ^
        // b --/
        let mut stations = Stations::new();
        let station_a = station(VisitStatus::VisitSuccess);
        let station_b = station(VisitStatus::VisitSuccess);
        let station_c = station(VisitStatus::NotReady);
        let node_index_a = stations.add_node(station_a);
        let node_index_b = stations.add_node(station_b);
        let node_index_c = stations.add_node(station_c);
        stations.add_edge(node_index_a, node_index_c, Workload::default())?;
        stations.add_edge(node_index_b, node_index_c, Workload::default())?;

        let station_c = stations.node_weight(node_index_c).unwrap();

        let visit_status_next =
            VisitStatusUpdater::visit_status_next(&stations, node_index_c, &station_c);

        assert_eq!(Some(VisitStatus::Queued), visit_status_next);
        Ok(())
    }

    #[test]
    fn updates_not_ready_to_parent_fail_when_any_parents_visit_fail()
    -> Result<(), WouldCycle<Workload>> {
        // a -> c
        //      ^
        // b --/
        let mut stations = Stations::new();
        let station_a = station(VisitStatus::VisitSuccess);
        let station_b = station(VisitStatus::VisitFail);
        let station_c = station(VisitStatus::NotReady);
        let node_index_a = stations.add_node(station_a);
        let node_index_b = stations.add_node(station_b);
        let node_index_c = stations.add_node(station_c);
        stations.add_edge(node_index_a, node_index_c, Workload::default())?;
        stations.add_edge(node_index_b, node_index_c, Workload::default())?;

        let station_c = stations.node_weight(node_index_c).unwrap();

        let visit_status_next =
            VisitStatusUpdater::visit_status_next(&stations, node_index_c, &station_c);

        assert_eq!(Some(VisitStatus::ParentFail), visit_status_next);
        Ok(())
    }

    #[test]
    fn updates_not_ready_to_parent_fail_when_any_parents_parent_fail()
    -> Result<(), WouldCycle<Workload>> {
        let mut stations = Stations::new();
        let station_a = station(VisitStatus::VisitSuccess);
        let station_b = station(VisitStatus::ParentFail);
        let station_c = station(VisitStatus::NotReady);
        let node_index_a = stations.add_node(station_a);
        let node_index_b = stations.add_node(station_b);
        let node_index_c = stations.add_node(station_c);
        stations.add_edge(node_index_a, node_index_c, Workload::default())?;
        stations.add_edge(node_index_b, node_index_c, Workload::default())?;

        let station_c = stations.node_weight(node_index_c).unwrap();

        let visit_status_next =
            VisitStatusUpdater::visit_status_next(&stations, node_index_c, &station_c);

        assert_eq!(Some(VisitStatus::ParentFail), visit_status_next);
        Ok(())
    }

    #[test]
    fn no_change_to_not_ready_when_any_parents_on_other_status() -> Result<(), WouldCycle<Workload>>
    {
        [
            VisitStatus::NotReady,
            VisitStatus::Queued,
            VisitStatus::InProgress,
        ]
        .iter()
        .copied()
        .try_for_each(|visit_status_parent| {
            let mut stations = Stations::new();
            let station_a = station(VisitStatus::VisitSuccess);
            let station_b = station(visit_status_parent);
            let station_c = station(VisitStatus::NotReady);
            let node_index_a = stations.add_node(station_a);
            let node_index_b = stations.add_node(station_b);
            let node_index_c = stations.add_node(station_c);
            stations.add_edge(node_index_a, node_index_c, Workload::default())?;
            stations.add_edge(node_index_b, node_index_c, Workload::default())?;

            let station_c = stations.node_weight(node_index_c).unwrap();

            let visit_status_next =
                VisitStatusUpdater::visit_status_next(&stations, node_index_c, &station_c);

            assert_eq!(None, visit_status_next);

            Ok(())
        })
    }

    #[test]
    fn no_change_to_parent_fail_visit_success_or_visit_fail() -> Result<(), WouldCycle<Workload>> {
        [
            VisitStatus::ParentFail,
            VisitStatus::VisitSuccess,
            VisitStatus::VisitFail,
        ]
        .iter()
        .copied()
        .try_for_each(|visit_status| {
            let mut stations = Stations::new();
            let station_a = station(visit_status);
            let node_index_a = stations.add_node(station_a);

            let station_a = stations.node_weight(node_index_a).unwrap();

            let visit_status_next =
                VisitStatusUpdater::visit_status_next(&stations, node_index_a, &station_a);

            assert_eq!(None, visit_status_next);

            Ok(())
        })
    }

    fn station(visit_status: VisitStatus) -> Station<()> {
        let station_spec = StationSpec::new(VisitFn(|_station| {
            Box::pin(async move { Result::<(), ()>::Ok(()) })
        }));
        Station::new(station_spec, visit_status)
    }
}

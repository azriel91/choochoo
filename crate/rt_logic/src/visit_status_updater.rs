use std::marker::PhantomData;

use choochoo_cfg_model::VisitStatus;
use choochoo_rt_model::{daggy::Walker, Destination, StationRtId};

/// Updates the [`VisitStatus`]es for all [`StationMut`]s.
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
/// `VisitFail` depending on [`StationSpec::visit`]'s result.
///
/// ## `VisitSuccess`
///
/// No transitions.
///
/// ## `VisitFail`
///
/// No transitions.
///
/// [`StationSpec::visit`]: crate::cfg_model::StationSpec::visit
/// [`StationMut`]: crate::rt_model::StationMut
/// [`Train::reach`]: crate::Train::reach
#[derive(Debug)]
pub struct VisitStatusUpdater<E> {
    /// Marker
    pub marker: PhantomData<E>,
}

impl<E> VisitStatusUpdater<E> {
    /// Updates the [`VisitStatus`]es for all [`StationMut`]s.
    ///
    /// `ParentFail` transitions are propagated through to all later stations,
    /// on the condition that the nodes are added in order.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` with all the stations and their progress
    ///   information.
    ///
    /// [`StationMut`]: crate::rt_model::StationMut
    pub fn update(dest: &Destination<E>) {
        let station_specs = dest.station_specs();
        let station_id_to_rt_id = dest.station_id_to_rt_id();

        station_specs.iter().for_each(|station_spec| {
            if let Some(station_rt_id) = station_id_to_rt_id.get(station_spec.id()) {
                let visit_status_next = Self::visit_status_next(dest, *station_rt_id);

                if let Some(visit_status_next) = visit_status_next {
                    let station_progress = dest
                        .station_progresses()
                        .get(station_rt_id)
                        .map(|station_progress| station_progress.borrow_mut());

                    if let Some(mut station_progress) = station_progress {
                        station_progress.visit_status = visit_status_next
                    }
                };
            }
        });
    }

    /// Updates the [`VisitStatus`]es for children of the given [`StationMut`].
    ///
    /// `ParentFail` transitions are propagated through to all later stations,
    /// on the condition that the nodes are added in order.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` with all the stations and their progress
    ///   information.
    /// * `station_rt_id`: Runtime ID of the parent station, whose children to
    ///   update.
    ///
    /// [`StationMut`]: crate::rt_model::StationMut
    pub fn update_children(dest: &Destination<E>, station_rt_id: StationRtId) {
        let station_specs = dest.station_specs();

        station_specs
            .children(station_rt_id)
            .iter(&*station_specs)
            .for_each(|(_edge, station_rt_id)| {
                let visit_status_next = Self::visit_status_next(dest, station_rt_id);

                if let Some(visit_status_next) = visit_status_next {
                    let station_progress = dest
                        .station_progresses()
                        .get(&station_rt_id)
                        .map(|station_progress| station_progress.borrow_mut());

                    if let Some(mut station_progress) = station_progress {
                        station_progress.visit_status = visit_status_next
                    }
                };
            });
    }

    /// Returns the [`VisitStatus`] to be transitioned to for a single
    /// [`StationMut`], if any.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` with all the stations and their progress
    ///   information.
    /// * `station_rt_id`: Runtime ID of the station whose next `VisitStatus` to
    ///   compute.
    ///
    /// [`StationMut`]: crate::rt_model::StationMut
    pub fn visit_status_next(
        dest: &Destination<E>,
        station_rt_id: StationRtId,
    ) -> Option<VisitStatus> {
        dest.station_progresses()
            .get(&station_rt_id)
            .and_then(|station_progress| station_progress.try_borrow())
            .and_then(|station_progress| {
                match station_progress.visit_status {
                    VisitStatus::NotReady => Self::transition_not_ready(dest, station_rt_id),
                    VisitStatus::Queued // TODO: Queued stations may need to transition to `NotReady`
                    | VisitStatus::CheckFail
                    | VisitStatus::InProgress
                    | VisitStatus::ParentFail
                    | VisitStatus::VisitSuccess
                    | VisitStatus::VisitUnnecessary
                    | VisitStatus::VisitFail => None,
                }
            })
    }

    fn transition_not_ready(
        dest: &Destination<E>,
        station_rt_id: StationRtId,
    ) -> Option<VisitStatus> {
        let station_specs = dest.station_specs();
        let station_progresses = dest.station_progresses();
        let station_id_to_rt_id = dest.station_id_to_rt_id();
        let visit_status_existing = station_progresses
            .get(&station_rt_id)
            .map(|station_progress| station_progress.borrow().visit_status);

        let parents_walker = station_specs.parents(station_rt_id);
        let visit_status_next = parents_walker
            .iter(station_specs)
            .filter_map(|(_, parent_station_rt_id)| station_specs.node_weight(parent_station_rt_id))
            .filter_map(|parent_station| {
                station_id_to_rt_id
                    .get(parent_station.id())
                    .and_then(|parent_station_rt_id| station_progresses.get(parent_station_rt_id))
            })
            .try_fold(
                Some(VisitStatus::Queued),
                |visit_status, parent_station_progress| {
                    if let Some(parent_station_progress) = parent_station_progress.try_borrow() {
                        match parent_station_progress.visit_status {
                            // If parent is already done, we keep going.
                            VisitStatus::VisitSuccess | VisitStatus::VisitUnnecessary => {}

                            // Short circuits:

                            // If parent / ancestor has failed, indicate it in this station.
                            VisitStatus::CheckFail
                            | VisitStatus::VisitFail
                            | VisitStatus::ParentFail => {
                                return Err(Some(VisitStatus::ParentFail));
                            }
                            // Don't change `VisitStatus` if parent is on any other `VisitStatus`.
                            VisitStatus::NotReady
                            | VisitStatus::Queued
                            | VisitStatus::InProgress => {
                                return Err(None);
                            }
                        }
                        Ok(visit_status)
                    } else {
                        // Parent is probably being processed.
                        Ok(visit_status_existing)
                    }
                },
            );

        match visit_status_next {
            Ok(visit_status_next) | Err(visit_status_next) => visit_status_next,
        }
    }
}

#[cfg(test)]
mod tests {
    use choochoo_cfg_model::{
        StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec, StationSpecFns,
        VisitStatus, Workload,
    };
    use choochoo_rt_model::{Destination, StationProgresses, StationRtId, StationSpecs};

    use super::VisitStatusUpdater;

    #[test]
    fn update_processes_all_possible_transitions() -> Result<(), Box<dyn std::error::Error>> {
        // a -> c
        //      ^
        // b --/
        //   \
        //    \--> d
        //
        // e --> f
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::VisitSuccess,
        )?;
        let station_b = add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::VisitSuccess,
        )?;
        let station_c = add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::NotReady,
        )?; // Should become `Queued`
        let station_d = add_station(
            &mut station_specs,
            &mut station_progresses,
            "d",
            VisitStatus::NotReady,
        )?; // Should become `Queued`
        let station_e = add_station(
            &mut station_specs,
            &mut station_progresses,
            "e",
            VisitStatus::VisitFail,
        )?;
        let station_f = add_station(
            &mut station_specs,
            &mut station_progresses,
            "f",
            VisitStatus::NotReady,
        )?; // Should become `ParentFail`
        station_specs.add_edge(station_a, station_c, Workload::default())?;
        station_specs.add_edge(station_b, station_c, Workload::default())?;
        station_specs.add_edge(station_b, station_d, Workload::default())?;
        station_specs.add_edge(station_e, station_f, Workload::default())?;
        let mut dest = Destination::new(station_specs, station_progresses);

        VisitStatusUpdater::update(&mut dest);

        let station_a = &dest.station_progresses()[&station_a];
        let station_b = &dest.station_progresses()[&station_b];
        let station_c = &dest.station_progresses()[&station_c];
        let station_d = &dest.station_progresses()[&station_d];
        let station_e = &dest.station_progresses()[&station_e];
        let station_f = &dest.station_progresses()[&station_f];
        assert_eq!(VisitStatus::VisitSuccess, station_a.borrow().visit_status);
        assert_eq!(VisitStatus::VisitSuccess, station_b.borrow().visit_status);
        assert_eq!(VisitStatus::Queued, station_c.borrow().visit_status);
        assert_eq!(VisitStatus::Queued, station_d.borrow().visit_status);
        assert_eq!(VisitStatus::VisitFail, station_e.borrow().visit_status);
        assert_eq!(VisitStatus::ParentFail, station_f.borrow().visit_status);
        Ok(())
    }

    #[test]
    fn update_propagates_parent_fail_transitions() -> Result<(), Box<dyn std::error::Error>> {
        // a -> c -> d -> e
        //      ^
        // b --/
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::InProgress,
        )?;
        let station_b = add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::VisitFail,
        )?;
        let station_c = add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::NotReady,
        )?;
        let station_d = add_station(
            &mut station_specs,
            &mut station_progresses,
            "d",
            VisitStatus::NotReady,
        )?;
        let station_e = add_station(
            &mut station_specs,
            &mut station_progresses,
            "e",
            VisitStatus::NotReady,
        )?;
        station_specs.add_edge(station_a, station_c, Workload::default())?;
        station_specs.add_edge(station_b, station_c, Workload::default())?;
        station_specs.add_edge(station_c, station_d, Workload::default())?;
        station_specs.add_edge(station_d, station_e, Workload::default())?;
        let mut dest = Destination::new(station_specs, station_progresses);

        VisitStatusUpdater::update(&mut dest);

        let station_a = &dest.station_progresses()[&station_a];
        let station_b = &dest.station_progresses()[&station_b];
        let station_c = &dest.station_progresses()[&station_c];
        let station_d = &dest.station_progresses()[&station_d];
        let station_e = &dest.station_progresses()[&station_e];
        assert_eq!(VisitStatus::InProgress, station_a.borrow().visit_status);
        assert_eq!(VisitStatus::VisitFail, station_b.borrow().visit_status);
        assert_eq!(VisitStatus::ParentFail, station_c.borrow().visit_status);
        assert_eq!(VisitStatus::ParentFail, station_d.borrow().visit_status);
        assert_eq!(VisitStatus::ParentFail, station_e.borrow().visit_status);
        Ok(())
    }

    #[test]
    fn updates_not_ready_to_queued_when_no_parents_exist() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_rt_id = add_station(
            &mut station_specs,
            &mut station_progresses,
            "n",
            VisitStatus::NotReady,
        )?;
        let dest = Destination::new(station_specs, station_progresses);

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_rt_id);

        assert_eq!(Some(VisitStatus::Queued), visit_status_next);
        Ok(())
    }

    #[test]
    fn updates_not_ready_to_queued_when_all_parents_visit_success()
    -> Result<(), Box<dyn std::error::Error>> {
        // a -> c
        //      ^
        // b --/
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::VisitSuccess,
        )?;
        let station_b = add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::VisitSuccess,
        )?;
        let station_c = add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::NotReady,
        )?;
        station_specs.add_edge(station_a, station_c, Workload::default())?;
        station_specs.add_edge(station_b, station_c, Workload::default())?;
        let dest = Destination::new(station_specs, station_progresses);

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

        assert_eq!(Some(VisitStatus::Queued), visit_status_next);
        Ok(())
    }

    #[test]
    fn updates_not_ready_to_queued_when_all_parents_visit_success_or_unnecessary()
    -> Result<(), Box<dyn std::error::Error>> {
        // a -> c
        //      ^
        // b --/
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::VisitSuccess,
        )?;
        let station_b = add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::VisitUnnecessary,
        )?;
        let station_c = add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::NotReady,
        )?;
        station_specs.add_edge(station_a, station_c, Workload::default())?;
        station_specs.add_edge(station_b, station_c, Workload::default())?;
        let dest = Destination::new(station_specs, station_progresses);

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

        assert_eq!(Some(VisitStatus::Queued), visit_status_next);
        Ok(())
    }

    #[test]
    fn updates_not_ready_to_parent_fail_when_any_parents_visit_fail()
    -> Result<(), Box<dyn std::error::Error>> {
        // a -> c
        //      ^
        // b --/
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::VisitSuccess,
        )?;
        let station_b = add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::VisitFail,
        )?;
        let station_c = add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::NotReady,
        )?;
        station_specs.add_edge(station_a, station_c, Workload::default())?;
        station_specs.add_edge(station_b, station_c, Workload::default())?;
        let dest = Destination::new(station_specs, station_progresses);

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

        assert_eq!(Some(VisitStatus::ParentFail), visit_status_next);
        Ok(())
    }

    #[test]
    fn updates_not_ready_to_parent_fail_when_any_parents_parent_fail()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::VisitSuccess,
        )?;
        let station_b = add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::ParentFail,
        )?;
        let station_c = add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::NotReady,
        )?;
        station_specs.add_edge(station_a, station_c, Workload::default())?;
        station_specs.add_edge(station_b, station_c, Workload::default())?;
        let dest = Destination::new(station_specs, station_progresses);

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

        assert_eq!(Some(VisitStatus::ParentFail), visit_status_next);
        Ok(())
    }

    #[test]
    fn no_change_to_not_ready_when_any_parents_on_other_status()
    -> Result<(), Box<dyn std::error::Error>> {
        [
            VisitStatus::NotReady,
            VisitStatus::Queued,
            VisitStatus::InProgress,
        ]
        .iter()
        .copied()
        .try_for_each(|visit_status_parent| {
            let mut station_specs = StationSpecs::new();
            let mut station_progresses = StationProgresses::new();
            let station_a = add_station(
                &mut station_specs,
                &mut station_progresses,
                "a",
                VisitStatus::VisitSuccess,
            )?;
            let station_b = add_station(
                &mut station_specs,
                &mut station_progresses,
                "b",
                visit_status_parent,
            )?;
            let station_c = add_station(
                &mut station_specs,
                &mut station_progresses,
                "c",
                VisitStatus::NotReady,
            )?;
            station_specs.add_edge(station_a, station_c, Workload::default())?;
            station_specs.add_edge(station_b, station_c, Workload::default())?;
            let dest = Destination::new(station_specs, station_progresses);

            let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

            assert_eq!(None, visit_status_next);

            Ok(())
        })
    }

    #[test]
    fn no_change_to_parent_fail_visit_success_or_visit_fail()
    -> Result<(), Box<dyn std::error::Error>> {
        [
            VisitStatus::ParentFail,
            VisitStatus::VisitSuccess,
            VisitStatus::VisitFail,
        ]
        .iter()
        .copied()
        .try_for_each(|visit_status| {
            let mut station_specs = StationSpecs::new();
            let mut station_progresses = StationProgresses::new();
            let station_a = add_station(
                &mut station_specs,
                &mut station_progresses,
                "a",
                visit_status,
            )?;
            let dest = Destination::new(station_specs, station_progresses);

            let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_a);

            assert_eq!(None, visit_status_next);

            Ok(())
        })
    }

    fn add_station(
        station_specs: &mut StationSpecs<()>,
        station_progresses: &mut StationProgresses,
        station_id: &'static str,
        visit_status: VisitStatus,
    ) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
        let name = String::from(station_id);
        let station_id = StationId::new(station_id)?;
        let station_spec_fns = {
            let visit_fn = StationFn::new(|_, _| Box::pin(async { Result::<(), ()>::Ok(()) }));
            StationSpecFns::new(visit_fn)
        };
        let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
        let station_progress = StationProgress::new(&station_spec, visit_status);
        let station_rt_id = station_specs.add_node(station_spec);

        station_progresses.insert(station_rt_id, station_progress);

        Ok(station_rt_id)
    }
}

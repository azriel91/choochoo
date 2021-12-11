use std::marker::PhantomData;

use choochoo_cfg_model::{
    daggy::Walker,
    rt::{StationRtId, VisitStatus},
};
use choochoo_rt_model::Destination;

/// Updates the [`VisitStatus`]es for all [`StationMutRef`]s.
///
/// The new visit status is calculated based on the station's visit result
/// and its parents' [`VisitStatus`]es.
///
/// # `VisitStatus` State Machine
///
/// ## `ParentPending` Stations
///
/// * If all parents are `VisitSuccess`, switch to `VisitQueued`.
/// * If at least one parent has `VisitFailed` or `ParentFail`, switch to
///   `ParentFail`.
///
/// ## `ParentFail` Stations
///
/// No transitions.
///
/// ## `VisitQueued` Stations
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
/// [`StationMutRef`]: crate::rt_model::StationMutRef
/// [`Train::reach`]: crate::Train::reach
#[derive(Debug)]
pub struct VisitStatusUpdater<E> {
    /// Marker
    pub marker: PhantomData<E>,
}

impl<E> VisitStatusUpdater<E>
where
    E: 'static,
{
    /// Updates the [`VisitStatus`]es for all [`StationMutRef`]s.
    ///
    /// `ParentFail` transitions are propagated through to all later stations,
    /// on the condition that the nodes are added in order.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` with all the stations and their progress
    ///   information.
    ///
    /// [`StationMutRef`]: crate::rt_model::StationMutRef
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

    /// Updates the [`VisitStatus`]es for children of the given
    /// [`StationMutRef`].
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
    /// [`StationMutRef`]: crate::rt_model::StationMutRef
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
    /// [`StationMutRef`], if any.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` with all the stations and their progress
    ///   information.
    /// * `station_rt_id`: Runtime ID of the station whose next `VisitStatus` to
    ///   compute.
    ///
    /// [`StationMutRef`]: crate::rt_model::StationMutRef
    pub fn visit_status_next(
        dest: &Destination<E>,
        station_rt_id: StationRtId,
    ) -> Option<VisitStatus> {
        dest.station_progresses()
            .get(&station_rt_id)
            .and_then(|station_progress| station_progress.try_borrow().ok())
            .and_then(|station_progress| {
                match station_progress.visit_status {
                    VisitStatus::SetupQueued => Self::transition_setup_queued(dest, station_rt_id),
                    VisitStatus::SetupSuccess => Some(Self::transition_setup_success(dest, station_rt_id)),
                    VisitStatus::ParentPending => Self::transition_parent_pending(dest, station_rt_id),
                    VisitStatus::VisitQueued // TODO: VisitQueued stations may need to transition to `ParentPending`
                    | VisitStatus::SetupFail
                    | VisitStatus::CheckFail
                    | VisitStatus::InProgress
                    | VisitStatus::ParentFail
                    | VisitStatus::VisitSuccess
                    | VisitStatus::VisitUnnecessary
                    | VisitStatus::VisitFail => None,
                }
            })
    }

    fn transition_setup_queued(
        dest: &Destination<E>,
        station_rt_id: StationRtId,
    ) -> Option<VisitStatus> {
        let station_specs = dest.station_specs();
        let station_progresses = dest.station_progresses();
        let station_id_to_rt_id = dest.station_id_to_rt_id();

        let parents_walker = station_specs.parents(station_rt_id);
        let visit_status_next = parents_walker
            .iter(station_specs)
            .filter_map(|(_, parent_station_rt_id)| station_specs.node_weight(parent_station_rt_id))
            .filter_map(|parent_station| {
                station_id_to_rt_id
                    .get(parent_station.id())
                    .and_then(|parent_station_rt_id| station_progresses.get(parent_station_rt_id))
            })
            .try_fold(None, |visit_status, parent_station_progress| {
                if let Ok(parent_station_progress) = parent_station_progress.try_borrow() {
                    match parent_station_progress.visit_status {
                        // If parent is already done, we keep checking other parents.
                        VisitStatus::SetupQueued | VisitStatus::SetupSuccess => {}

                        // Short circuits:

                        // If parent / ancestor has failed, indicate it in this station.
                        VisitStatus::SetupFail | VisitStatus::ParentFail => {
                            return Err(Some(VisitStatus::ParentFail));
                        }
                        // Don't change `VisitStatus` if parent is on any other `VisitStatus`.
                        VisitStatus::CheckFail
                        | VisitStatus::VisitQueued
                        | VisitStatus::VisitFail
                        | VisitStatus::ParentPending
                        | VisitStatus::VisitUnnecessary
                        | VisitStatus::VisitSuccess
                        | VisitStatus::InProgress => unreachable!(
                            "Parent station status should not be {:?} during setup phase. This is a bug.",
                            parent_station_progress.visit_status
                        ),
                    }
                    Ok(visit_status)
                } else {
                    // Parent is probably being processed.
                    Ok(None)
                }
            });

        match visit_status_next {
            Ok(visit_status_next) | Err(visit_status_next) => visit_status_next,
        }
    }

    fn transition_setup_success(dest: &Destination<E>, station_rt_id: StationRtId) -> VisitStatus {
        let station_specs = dest.station_specs();
        let parents_walker = station_specs.parents(station_rt_id);
        if parents_walker.iter(station_specs).next().is_some() {
            VisitStatus::ParentPending
        } else {
            VisitStatus::VisitQueued
        }
    }

    fn transition_parent_pending(
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
                Some(VisitStatus::VisitQueued),
                |visit_status, parent_station_progress| {
                    if let Ok(parent_station_progress) = parent_station_progress.try_borrow() {
                        match parent_station_progress.visit_status {
                            // If parent is already done, we keep checking other parents.
                            VisitStatus::VisitSuccess | VisitStatus::VisitUnnecessary => {}

                            // Short circuits:

                            // If parent / ancestor has failed, indicate it in this station.
                            VisitStatus::CheckFail
                            | VisitStatus::VisitFail
                            | VisitStatus::ParentFail => {
                                return Err(Some(VisitStatus::ParentFail));
                            }
                            // Don't change `VisitStatus` if parent is on any other `VisitStatus`.
                            VisitStatus::ParentPending
                            | VisitStatus::VisitQueued
                            | VisitStatus::InProgress => {
                                return Err(None);
                            }

                            VisitStatus::SetupQueued
                            | VisitStatus::SetupSuccess
                            | VisitStatus::SetupFail => unreachable!(
                                "Parent station status should not be {:?} during visit phase. This is a bug.",
                                parent_station_progress.visit_status
                            )
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

use std::marker::PhantomData;

use choochoo_cfg_model::{
    daggy::Walker,
    rt::{OpStatus, StationRtId},
};
use choochoo_rt_model::Destination;

/// Updates the [`OpStatus`]es for all [`StationMutRef`]s.
///
/// The new op status is calculated based on the station's visit result
/// and its parents' [`OpStatus`]es.
///
/// # `OpStatus` State Machine
///
/// ## `ParentPending` Stations
///
/// * If all parents are `WorkSuccess`, switch to `OpQueued`.
/// * If at least one parent has `WorkFailed` or `ParentFail`, switch to
///   `ParentFail`.
///
/// ## `ParentFail` Stations
///
/// No transitions.
///
/// ## `OpQueued` Stations
///
/// No transitions -- [`Train::reach`] sets this to `WorkInProgress` when
/// visiting the station.
///
/// ## `WorkInProgress` Stations
///
/// No transitions -- [`Train::reach`] sets this to `WorkSuccess` or
/// `WorkFail` depending on [`StationMutRef::visit`]'s result.
///
/// ## `WorkSuccess`
///
/// No transitions.
///
/// ## `WorkFail`
///
/// No transitions.
///
/// [`StationMutRef::visit`]: crate::cfg_model::rt::StationMutRef::visit
/// [`StationMutRef`]: crate::cfg_model::rt::StationMutRef
/// [`Train::reach`]: crate::Train::reach
#[derive(Debug)]
pub struct OpStatusUpdater<E> {
    /// Marker
    pub marker: PhantomData<E>,
}

impl<E> OpStatusUpdater<E>
where
    E: 'static,
{
    /// Updates the [`OpStatus`]es for all [`StationMutRef`]s.
    ///
    /// `ParentFail` transitions are propagated through to all later stations,
    /// on the condition that the nodes are added in order.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` with all the stations and their progress
    ///   information.
    ///
    /// [`StationMutRef`]: crate::cfg_model::rt::StationMutRef
    pub fn update(dest: &Destination<E>) {
        let station_specs = dest.station_specs();
        let station_id_to_rt_id = dest.station_id_to_rt_id();

        station_specs.iter().for_each(|station_spec| {
            if let Some(station_rt_id) = station_id_to_rt_id.get(station_spec.id()) {
                let op_status_next = Self::op_status_next(dest, *station_rt_id);

                if let Some(op_status_next) = op_status_next {
                    let station_progress = dest
                        .station_progresses()
                        .get(station_rt_id)
                        .map(|station_progress| station_progress.borrow_mut());

                    if let Some(mut station_progress) = station_progress {
                        station_progress.op_status = op_status_next
                    }
                };
            }
        });
    }

    /// Updates the [`OpStatus`]es for children of the given
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
    /// [`StationMutRef`]: crate::cfg_model::rt::StationMutRef
    pub fn update_children(dest: &Destination<E>, station_rt_id: StationRtId) {
        let station_specs = dest.station_specs();

        station_specs
            .children(station_rt_id)
            .iter(station_specs)
            .for_each(|(_edge, station_rt_id)| {
                let op_status_next = Self::op_status_next(dest, station_rt_id);

                if let Some(op_status_next) = op_status_next {
                    let station_progress = dest
                        .station_progresses()
                        .get(&station_rt_id)
                        .map(|station_progress| station_progress.borrow_mut());

                    if let Some(mut station_progress) = station_progress {
                        station_progress.op_status = op_status_next
                    }
                };
            });
    }

    /// Returns the [`OpStatus`] to be transitioned to for a single
    /// [`StationMutRef`], if any.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` with all the stations and their progress
    ///   information.
    /// * `station_rt_id`: Runtime ID of the station whose next `OpStatus` to
    ///   compute.
    ///
    /// [`StationMutRef`]: crate::cfg_model::rt::StationMutRef
    pub fn op_status_next(dest: &Destination<E>, station_rt_id: StationRtId) -> Option<OpStatus> {
        dest.station_progresses()
            .get(&station_rt_id)
            .and_then(|station_progress| station_progress.try_borrow().ok())
            .and_then(|station_progress| {
                match station_progress.op_status {
                    OpStatus::SetupQueued => Self::transition_setup_queued(dest, station_rt_id),
                    OpStatus::SetupSuccess => Some(Self::transition_setup_success(dest, station_rt_id)),
                    OpStatus::ParentPending => Self::transition_parent_pending(dest, station_rt_id),
                    OpStatus::OpQueued // TODO: OpQueued stations may need to transition to `ParentPending`
                    | OpStatus::SetupFail
                    | OpStatus::CheckFail
                    | OpStatus::WorkInProgress
                    | OpStatus::ParentFail
                    | OpStatus::WorkSuccess
                    | OpStatus::WorkUnnecessary
                    | OpStatus::WorkFail => None,
                }
            })
    }

    fn transition_setup_queued(
        dest: &Destination<E>,
        station_rt_id: StationRtId,
    ) -> Option<OpStatus> {
        let station_specs = dest.station_specs();
        let station_progresses = dest.station_progresses();
        let station_id_to_rt_id = dest.station_id_to_rt_id();

        let parents_walker = station_specs.parents(station_rt_id);
        let op_status_next = parents_walker
            .iter(station_specs)
            .filter_map(|(_, parent_station_rt_id)| station_specs.node_weight(parent_station_rt_id))
            .filter_map(|parent_station| {
                station_id_to_rt_id
                    .get(parent_station.id())
                    .and_then(|parent_station_rt_id| station_progresses.get(parent_station_rt_id))
            })
            .try_fold(None, |op_status, parent_station_progress| {
                if let Ok(parent_station_progress) = parent_station_progress.try_borrow() {
                    match parent_station_progress.op_status {
                        // If parent is already done, we keep checking other parents.
                        OpStatus::SetupQueued | OpStatus::SetupSuccess => {}

                        // Short circuits:

                        // If parent / ancestor has failed, indicate it in this station.
                        OpStatus::SetupFail | OpStatus::ParentFail => {
                            return Err(Some(OpStatus::ParentFail));
                        }
                        // Don't change `OpStatus` if parent is on any other `OpStatus`.
                        OpStatus::CheckFail
                        | OpStatus::OpQueued
                        | OpStatus::WorkFail
                        | OpStatus::ParentPending
                        | OpStatus::WorkUnnecessary
                        | OpStatus::WorkSuccess
                        | OpStatus::WorkInProgress => unreachable!(
                            "Parent station status should not be {:?} during setup phase. This is a bug.",
                            parent_station_progress.op_status
                        ),
                    }
                    Ok(op_status)
                } else {
                    // Parent is probably being processed.
                    Ok(None)
                }
            });

        match op_status_next {
            Ok(op_status_next) | Err(op_status_next) => op_status_next,
        }
    }

    fn transition_setup_success(dest: &Destination<E>, station_rt_id: StationRtId) -> OpStatus {
        let station_specs = dest.station_specs();
        let parents_walker = station_specs.parents(station_rt_id);
        if parents_walker.iter(station_specs).next().is_some() {
            OpStatus::ParentPending
        } else {
            OpStatus::OpQueued
        }
    }

    fn transition_parent_pending(
        dest: &Destination<E>,
        station_rt_id: StationRtId,
    ) -> Option<OpStatus> {
        let station_specs = dest.station_specs();
        let station_progresses = dest.station_progresses();
        let station_id_to_rt_id = dest.station_id_to_rt_id();
        let op_status_existing = station_progresses
            .get(&station_rt_id)
            .map(|station_progress| station_progress.borrow().op_status);

        let parents_walker = station_specs.parents(station_rt_id);
        let op_status_next = parents_walker
            .iter(station_specs)
            .filter_map(|(_, parent_station_rt_id)| station_specs.node_weight(parent_station_rt_id))
            .filter_map(|parent_station| {
                station_id_to_rt_id
                    .get(parent_station.id())
                    .and_then(|parent_station_rt_id| station_progresses.get(parent_station_rt_id))
            })
            .try_fold(
                Some(OpStatus::OpQueued),
                |op_status, parent_station_progress| {
                    if let Ok(parent_station_progress) = parent_station_progress.try_borrow() {
                        match parent_station_progress.op_status {
                            // If parent is already done, we keep checking other parents.
                            OpStatus::WorkSuccess | OpStatus::WorkUnnecessary => {}

                            // Short circuits:

                            // If parent / ancestor has failed, indicate it in this station.
                            OpStatus::CheckFail
                            | OpStatus::WorkFail
                            | OpStatus::ParentFail => {
                                return Err(Some(OpStatus::ParentFail));
                            }
                            // Don't change `OpStatus` if parent is on any other `OpStatus`.
                            OpStatus::ParentPending
                            | OpStatus::OpQueued
                            | OpStatus::WorkInProgress => {
                                return Err(None);
                            }

                            OpStatus::SetupQueued
                            | OpStatus::SetupSuccess
                            | OpStatus::SetupFail => unreachable!(
                                "Parent station status should not be {:?} during visit phase. This is a bug.",
                                parent_station_progress.op_status
                            )
                        }
                        Ok(op_status)
                    } else {
                        // Parent is probably being processed.
                        Ok(op_status_existing)
                    }
                },
            );

        match op_status_next {
            Ok(op_status_next) | Err(op_status_next) => op_status_next,
        }
    }
}

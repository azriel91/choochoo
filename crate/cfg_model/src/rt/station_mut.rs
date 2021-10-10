use rt_map::RefMut;

use crate::{
    rt::{StationProgress, StationRtId},
    StationSpec,
};

/// Station runtime information.
///
/// This includes a mutable reference to the station's progress, so it can only
/// be constructed when nothing else has a reference to this station's progress.
#[derive(Debug)]
pub struct StationMut<'s, E> {
    /// Behaviour specification of the station.
    pub spec: &'s StationSpec<E>,
    /// Runtime identifier for a station.
    pub rt_id: StationRtId,
    /// Station progress to reaching the destination.
    pub progress: RefMut<'s, StationProgress>,
}

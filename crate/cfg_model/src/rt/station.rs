use rt_map::Ref;

use crate::{
    rt::{StationProgress, StationRtId},
    StationSpec,
};

/// Station runtime information.
///
/// This includes an immutable reference to the station's progress.
#[derive(Debug)]
pub struct Station<'s, E> {
    /// Behaviour specification of the station.
    pub spec: &'s StationSpec<E>,
    /// Runtime identifier for a station.
    pub rt_id: StationRtId,
    /// Station progress to reaching the destination.
    pub progress: Ref<'s, StationProgress>,
}

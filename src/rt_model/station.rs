use rt_map::RefMut;

use crate::{
    cfg_model::{StationProgress, StationSpec},
    rt_model::StationRtId,
};

/// Station runtime information.
#[derive(Debug)]
pub struct Station<'s, E> {
    /// Behaviour specification of the station.
    pub spec: &'s StationSpec<E>,
    /// Runtime identifier for a station.
    pub rt_id: StationRtId,
    /// Station progress to reaching the destination.
    pub progress: RefMut<'s, StationProgress<E>>,
}

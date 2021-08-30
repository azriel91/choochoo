use std::ops::{Deref, DerefMut};

use rt_map::RtMap;

use crate::{cfg_model::StationProgress, rt_model::StationRtId};

/// Map from [`StationId`] to the runtime data.
#[derive(Debug, Default)]
pub struct StationProgresses<E>(pub RtMap<StationRtId, StationProgress<E>>);

impl<E> StationProgresses<E> {
    /// Returns an empty `StationProgresses` map.
    pub fn new() -> Self {
        Self(RtMap::new())
    }

    /// Creates an empty `StationProgresses` map with the specified capacity.
    ///
    /// The map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the map will not allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(RtMap::with_capacity(capacity))
    }
}

impl<E> Deref for StationProgresses<E> {
    type Target = RtMap<StationRtId, StationProgress<E>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for StationProgresses<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

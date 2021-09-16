use std::ops::{Deref, DerefMut};

use rt_map::RtMap;

use choochoo_cfg_model::StationProgress;

use crate::StationRtId;

/// Map from [`StationRtId`] to the runtime data.
#[derive(Debug, Default)]
pub struct StationProgresses(pub RtMap<StationRtId, StationProgress>);

impl StationProgresses {
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

impl Deref for StationProgresses {
    type Target = RtMap<StationRtId, StationProgress>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StationProgresses {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

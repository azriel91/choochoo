use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use choochoo_cfg_model::rt::{StationDir, StationRtId};

/// Map from [`StationRtId`] to each station's execution directory.
#[derive(Clone, Debug, Default)]
pub struct StationDirs(pub HashMap<StationRtId, StationDir>);

impl StationDirs {
    /// Returns an empty `StationDirs` map.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Creates an empty `StationDirs` map with the specified capacity.
    ///
    /// The map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the map will not allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }
}

impl Deref for StationDirs {
    type Target = HashMap<StationRtId, StationDir>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StationDirs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

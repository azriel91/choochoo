use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use choochoo_cfg_model::rt::StationRtId;

use crate::ProfileHistoryStationDir;

/// Map from [`StationRtId`] to each station's history directory.
#[derive(Clone, Debug, Default)]
pub struct ProfileHistoryStationDirs(pub HashMap<StationRtId, ProfileHistoryStationDir>);

impl ProfileHistoryStationDirs {
    /// Returns an empty `ProfileHistoryStationDirs` map.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Creates an empty `ProfileHistoryStationDirs` map with the specified
    /// capacity.
    ///
    /// The map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the map will not allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }
}

impl Deref for ProfileHistoryStationDirs {
    type Target = HashMap<StationRtId, ProfileHistoryStationDir>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProfileHistoryStationDirs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

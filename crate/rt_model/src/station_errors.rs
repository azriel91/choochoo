use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use indexmap::IndexMap;
use tokio::sync::RwLock;

use crate::StationRtId;

/// Errors encountered when visiting stations.
#[derive(Clone, Debug)]
pub struct StationErrors<E>(Arc<RwLock<IndexMap<StationRtId, E>>>);

impl<E> StationErrors<E> {
    /// Returns new [`StationErrors`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<E> Default for StationErrors<E> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(IndexMap::new())))
    }
}

impl<E> Deref for StationErrors<E> {
    type Target = Arc<RwLock<IndexMap<StationRtId, E>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for StationErrors<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

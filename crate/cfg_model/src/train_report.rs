use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use resman::{Ref, Resources};

use crate::{RwFiles, StationErrors};

/// Record of what happened during a train's drive.
#[derive(Debug)]
pub struct TrainReport<E>(Resources, PhantomData<E>);

impl<E> TrainReport<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    /// Returns a new TrainReport.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to [`StationErrors`].
    ///
    /// There is `errors_mut` as [`StationErrors`] is behind a [`RwLock`], and
    /// you can choose to [`read`] or [`write`] as necessary.
    ///
    /// [`RwLock`]: tokio::sync::RwLock
    /// [`read`]: tokio::sync::RwLock::read
    /// [`write`]: tokio::sync::RwLock::write
    pub fn station_errors(&self) -> Ref<StationErrors<E>> {
        self.0.borrow::<StationErrors<E>>()
    }
}

impl<E> Default for TrainReport<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        let mut resources = Resources::default();
        resources.insert(RwFiles::new());
        resources.insert(StationErrors::<E>::new());

        Self(resources, PhantomData)
    }
}

impl<E> Deref for TrainReport<E> {
    type Target = Resources;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for TrainReport<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

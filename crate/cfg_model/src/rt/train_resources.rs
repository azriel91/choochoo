use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use choochoo_resource::FilesRw;
use resman::{Ref, Resources};

use crate::rt::StationErrors;

/// Record of what happened during a train's drive.
#[derive(Debug)]
pub struct TrainResources<E>(Resources, PhantomData<E>);

impl<E> TrainResources<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    /// Returns a new TrainResources.
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

impl<E> Default for TrainResources<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        let mut resources = Resources::default();
        resources.insert(FilesRw::new());
        resources.insert(StationErrors::<E>::new());

        Self(resources, PhantomData)
    }
}

impl<E> Deref for TrainResources<E> {
    type Target = Resources;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for TrainResources<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

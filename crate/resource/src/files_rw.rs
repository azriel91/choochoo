use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::Files;

/// Atomic RW access to `Files`.
#[derive(Debug, Default)]
pub struct FilesRw(Arc<RwLock<Files>>);

impl FilesRw {
    /// Returns new [`FilesRw`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for FilesRw {
    type Target = Arc<RwLock<Files>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FilesRw {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

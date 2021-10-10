use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::RwLock;

/// Stores source data strings for [`codespan`] to render.
///
/// [`codespan`]: srcerr::codespan
pub type Files = srcerr::codespan::Files<Cow<'static, str>>;

/// Atomic RW access to `Files`.
#[derive(Debug, Default)]
pub struct RwFiles(Arc<RwLock<Files>>);

impl RwFiles {
    /// Returns new [`RwFiles`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for RwFiles {
    type Target = Arc<RwLock<Files>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RwFiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

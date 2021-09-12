use std::{borrow::Cow, sync::Arc};

use tokio::sync::RwLock;

/// Stores source data strings for [`codespan`] to render.
///
/// [`codespan`]: srcerr::codespan
pub type Files = srcerr::codespan::Files<Cow<'static, str>>;

/// Atomic RW access to `Files`.
pub type RwFiles = Arc<RwLock<Files>>;

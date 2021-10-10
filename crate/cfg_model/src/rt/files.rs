use std::borrow::Cow;

/// Stores source data strings for [`codespan`] to render.
///
/// [`codespan`]: srcerr::codespan
pub type Files = srcerr::codespan::Files<Cow<'static, str>>;

use std::borrow::Cow;

/// Stores source data strings for [`codespan`] to render.
pub type Files = srcerr::codespan::Files<Cow<'static, str>>;

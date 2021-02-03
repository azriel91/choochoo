use std::borrow::Cow;

use srcerr::codespan_reporting::{diagnostic::Diagnostic, files::Files};

/// Types that can be represented as a [`Diagnostic`].
pub trait AsDiagnostic<'files> {
    /// Type of `FileId` in [`srcerr::codespan_reporting::Files`].
    type Files: Files<'files>;

    /// Returns the information in this type as a `Diagnostic`.
    ///
    /// This can be used in [`term::emit`] to render to the console.
    ///
    /// [`term::emit`]: srcerr::codespan_reporting::term::emit
    fn as_diagnostic(
        &self,
        files: &'files Self::Files,
    ) -> Diagnostic<<Self::Files as Files<'files>>::FileId>;
}

impl<'a> AsDiagnostic<'a> for () {
    type Files = codespan::Files<Cow<'a, str>>;

    fn as_diagnostic(
        &self,
        _files: &'a Self::Files,
    ) -> Diagnostic<<Self::Files as Files<'a>>::FileId> {
        Diagnostic::error()
    }
}

use std::borrow::Cow;

use srcerr::{
    codespan_reporting::{diagnostic::Diagnostic, files::Files},
    ErrorCode, ErrorDetail, SourceError,
};

/// Types that can be represented as a [`Diagnostic`].
pub trait AsDiagnostic<'files> {
    /// Type of `FileId` in [`srcerr::codespan_reporting::files::Files`].
    type Files: Files<'files>;

    /// Returns the information in this type as a `Diagnostic`.
    ///
    /// This can be used in [`term::emit`] to render to the console.
    ///
    /// [`term::emit`]: srcerr::codespan_reporting::term::emit
    fn as_diagnostic(
        &self,
        files: &Self::Files,
    ) -> Diagnostic<<Self::Files as Files<'files>>::FileId>;
}

impl<'a> AsDiagnostic<'a> for () {
    type Files = srcerr::codespan::Files<Cow<'a, str>>;

    fn as_diagnostic(
        &self,
        _files: &Self::Files,
    ) -> Diagnostic<<Self::Files as Files<'a>>::FileId> {
        Diagnostic::error()
    }
}

impl<'files, Ec, Ed, Fs> AsDiagnostic<'files> for SourceError<'files, Ec, Ed, Fs>
where
    Ec: ErrorCode,
    Ed: ErrorDetail<'files, Files = Fs>,
    Fs: Files<'files>,
{
    type Files = Fs;

    fn as_diagnostic(
        &self,
        files: &Self::Files,
    ) -> Diagnostic<<Self::Files as Files<'files>>::FileId> {
        SourceError::as_diagnostic(self, files)
    }
}

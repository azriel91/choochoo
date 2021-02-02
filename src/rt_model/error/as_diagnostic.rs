use srcerr::codespan_reporting::diagnostic::Diagnostic;

/// Types that can be represented as a [`Diagnostic`].
pub trait AsDiagnostic {
    /// Type of `FileId` in [`srcerr::codespan_reporting::Files`].
    type FileId;

    /// Returns the information in this type as a `Diagnostic`.
    ///
    /// This can be used in [`term::emit`] to render to the console.
    ///
    /// [`term::emit`]: srcerr::codespan_reporting::term::emit
    fn as_diagnostic(&self) -> Diagnostic<Self::FileId>;
}

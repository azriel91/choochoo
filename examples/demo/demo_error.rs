use std::fmt;

use choochoo::{
    cfg_model::{
        srcerr::{
            self,
            codespan_reporting::diagnostic::{Diagnostic, Severity},
            SourceError,
        },
        Files,
    },
    rt_model::error::StationSpecError,
};

use crate::{ErrorCode, ErrorDetail};

#[derive(Debug)]
pub struct DemoError(pub SourceError<'static, ErrorCode, ErrorDetail, Files>);

impl DemoError {
    pub fn new(code: ErrorCode, detail: ErrorDetail, severity: Severity) -> Self {
        Self(SourceError::new(code, detail, severity))
    }
}

impl choochoo::rt_model::error::AsDiagnostic<'static> for DemoError {
    type Files = Files;

    fn as_diagnostic(
        &self,
        files: &Self::Files,
    ) -> Diagnostic<<Self::Files as srcerr::codespan_reporting::files::Files<'static>>::FileId>
    {
        SourceError::as_diagnostic(&self.0, files)
    }
}

impl From<StationSpecError> for DemoError {
    fn from(error: StationSpecError) -> DemoError {
        let code = ErrorCode::StationSpecError;
        let detail = ErrorDetail::StationSpecError(error);

        DemoError::new(code, detail, Severity::Bug)
    }
}

impl fmt::Display for DemoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for DemoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

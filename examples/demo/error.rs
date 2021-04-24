use std::borrow::Cow;

use codespan::{FileId, Files, Span};
use srcerr::codespan_reporting::diagnostic::Label;

/// Error codes for simple example.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    /// Error when a value is out of range.
    ArtifactServerConnect,
}

impl srcerr::ErrorCode for ErrorCode {
    const ERROR_CODE_MAX: usize = 2;
    const PREFIX: &'static str = "E";

    fn code(self) -> usize {
        match self {
            Self::ArtifactServerConnect => 1,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::ArtifactServerConnect => "Failed to connect to server to upload app.zip.",
        }
    }
}

/// Error detail for simple example.
#[derive(Debug)]
pub enum ErrorDetail {
    /// Error when a value is out of range.
    ArtifactServerConnect {
        /// Artifact server address file ID.
        address_file_id: FileId,
        /// Span of the full socket address.
        address_span: Span,
        /// Span of the host address.
        host_span: Span,
        /// Span of the port.
        port_span: Span,
        /// Underlying IO error.
        error: reqwest::Error,
    },
}

impl<'files> srcerr::ErrorDetail<'files> for ErrorDetail {
    type Files = Files<Cow<'files, str>>;

    fn labels(&self) -> Vec<Label<FileId>> {
        match self {
            Self::ArtifactServerConnect {
                address_file_id,
                address_span,
                ..
            } => {
                vec![
                    Label::primary(*address_file_id, *address_span)
                        .with_message("failed to connect to server"),
                ]
            }
        }
    }

    fn notes(&self, files: &Self::Files) -> Vec<String> {
        match self {
            Self::ArtifactServerConnect {
                address_file_id,
                host_span,
                port_span,
                ..
            } => {
                let host = files
                    .source_slice(*address_file_id, *host_span)
                    .expect("Expected file to exist.");
                let port = files
                    .source_slice(*address_file_id, *port_span)
                    .expect("Expected file to exist.");
                vec![format!(
                    "Try running `simple-http-server --nocache -u --ip {host} --port {port}`.",
                    host = host,
                    port = port
                )]
            }
        }
    }
}

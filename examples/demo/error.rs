use std::borrow::Cow;

use srcerr::{
    codespan::{FileId, Files, Span},
    codespan_reporting::diagnostic::Label,
};

/// Error codes for simple example.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    /// Error when Opening "app.zip".
    AppZipOpen,
    /// Error when connecting to the artifact server.
    ArtifactServerConnect,
}

impl srcerr::ErrorCode for ErrorCode {
    const ERROR_CODE_MAX: usize = 2;
    const PREFIX: &'static str = "E";

    fn code(self) -> usize {
        match self {
            Self::AppZipOpen => 1,
            Self::ArtifactServerConnect => 2,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::AppZipOpen => "Failed to open `app.zip` to upload.",
            Self::ArtifactServerConnect => "Failed to connect to server to upload app.zip.",
        }
    }
}

/// Error detail for demo.
#[derive(Debug)]
pub enum ErrorDetail {
    /// Error when connecting to the artifact server.
    AppZipOpen {
        /// `app.zip` path file ID.
        app_zip_path_file_id: FileId,
        /// Span of the app.zip path.
        app_zip_path_span: Span,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Error when connecting to the artifact server.
    ArtifactServerConnect {
        /// Artifact server address file ID.
        address_file_id: FileId,
        /// Span of the full socket address.
        address_span: Span,
        /// Span of the host address.
        host_span: Span,
        /// Span of the port.
        port_span: Span,
        /// Underlying `reqwest` error.
        error: reqwest::Error,
    },
}

impl<'files> srcerr::ErrorDetail<'files> for ErrorDetail {
    type Files = Files<Cow<'files, str>>;

    fn labels(&self) -> Vec<Label<FileId>> {
        match self {
            Self::AppZipOpen {
                app_zip_path_file_id,
                app_zip_path_span,
                ..
            } => {
                vec![
                    Label::secondary(*app_zip_path_file_id, *app_zip_path_span)
                        .with_message("failed to open file"),
                ]
            }
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
            Self::AppZipOpen {
                app_zip_path_file_id,
                app_zip_path_span,
                ..
            } => {
                let app_zip_path = files
                    .source_slice(*app_zip_path_file_id, *app_zip_path_span)
                    .expect("Expected file to exist.");
                vec![format!(
                    "Try running `ls -l {app_zip_path}` to check file existence and permissions.",
                    app_zip_path = app_zip_path
                )]
            }
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

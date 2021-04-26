use std::borrow::Cow;

use srcerr::{
    codespan::{FileId, Files, Span},
    codespan_reporting::diagnostic::Label,
};

/// Error codes for simple example.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    /// Error when opening `app.zip`.
    AppZipOpen,
    /// Error when connecting to the artifact server.
    ArtifactServerConnect,
    /// Artifact server rejected `app.zip`.
    AppZipReject,
    /// Failed to create application database.
    DatabaseCreate,
}

impl srcerr::ErrorCode for ErrorCode {
    const ERROR_CODE_MAX: usize = 10;
    const PREFIX: &'static str = "E";

    fn code(self) -> usize {
        match self {
            Self::AppZipOpen => 1,
            Self::ArtifactServerConnect => 2,
            Self::AppZipReject => 3,
            Self::DatabaseCreate => 4,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::AppZipOpen => "Failed to open `app.zip` to upload.",
            Self::ArtifactServerConnect => "Failed to connect to server to upload `app.zip`.",
            Self::AppZipReject => "Artifact server rejected `app.zip`.",
            Self::DatabaseCreate => "Failed to create application database.",
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
    /// Error when the artifact server has rejected app.zip.
    AppZipReject {
        /// `app.zip` path file ID.
        app_zip_path_file_id: FileId,
        /// Span of the app.zip path.
        app_zip_path_span: Span,
        /// Artifact server address file ID.
        address_file_id: FileId,
        /// Span of the full socket address.
        address_span: Span,
        /// Reason provided by the server.
        server_message: Option<String>,
    },
    /// Failed to create application database.
    DatabaseCreate {
        /// Database name file ID.
        db_name_file_id: FileId,
        /// Span of the database name.
        db_name_span: Span,
        /// Underlying IO error.
        error: std::io::Error,
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
            Self::AppZipReject {
                app_zip_path_file_id,
                app_zip_path_span,
                address_file_id,
                address_span,
                ..
            } => {
                vec![
                    Label::secondary(*address_file_id, *address_span)
                        .with_message("server address"),
                    Label::secondary(*app_zip_path_file_id, *app_zip_path_span)
                        .with_message("file exists here"),
                ]
            }
            Self::DatabaseCreate {
                db_name_file_id,
                db_name_span,
                ..
            } => {
                vec![
                    Label::primary(*db_name_file_id, *db_name_span)
                        .with_message("failed to create database"),
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
            Self::AppZipReject {
                app_zip_path_file_id,
                app_zip_path_span,
                server_message,
                ..
            } => {
                let app_zip_path = files
                    .source_slice(*app_zip_path_file_id, *app_zip_path_span)
                    .expect("Expected file to exist.");
                let zip_valid_hint = format!(
                    "Check that `{app_zip_path}` is a valid zip file.",
                    app_zip_path = app_zip_path
                );

                if let Some(server_message) = server_message.as_deref() {
                    let server_message_hint = format!("Message from server:\n{}", server_message);
                    vec![zip_valid_hint, server_message_hint]
                } else {
                    vec![zip_valid_hint]
                }
            }
            Self::DatabaseCreate { .. } => {
                vec![]
            }
        }
    }
}

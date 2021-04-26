use std::{borrow::Cow, fmt};

use srcerr::{
    codespan::{FileId, Files, Span},
    codespan_reporting::diagnostic::Label,
    ErrorCode as _,
};

/// Error codes for simple example.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    /// Failed to open `app.zip` to upload.
    AppZipOpen,
    /// Failed to connect to artifact server.
    ArtifactServerConnect,
    /// Artifact server rejected `app.zip`.
    AppZipReject,
    /// Failed to create application database.
    DatabaseCreate,
    /// Application server failed to get `app.zip`.
    AppZipDownload,
    /// `app.zip` download connection broke.
    AppZipStream,
    /// Web server failed to write `app.zip` to disk.
    AppZipWrite,
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
            Self::AppZipDownload => 5,
            Self::AppZipStream => 6,
            Self::AppZipWrite => 7,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::AppZipOpen => "Failed to open `app.zip` to upload.",
            Self::ArtifactServerConnect => "Failed to connect to artifact server.",
            Self::AppZipReject => "Artifact server rejected `app.zip`.",
            Self::DatabaseCreate => "Failed to create application database.",
            Self::AppZipDownload => "Application server failed to get `app.zip`.",
            Self::AppZipStream => "`app.zip` download connection broke.",
            Self::AppZipWrite => "Web server failed to write `app.zip` to disk.",
        }
    }
}

/// Error detail for demo.
#[derive(Debug)]
pub enum ErrorDetail {
    /// Failed to open `app.zip` to upload.
    AppZipOpen {
        /// `app.zip` path file ID.
        app_zip_path_file_id: FileId,
        /// Span of the app.zip path.
        app_zip_path_span: Span,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Failed to connect to artifact server.
    ArtifactServerConnect {
        /// Artifact server address file ID.
        address_file_id: FileId,
        /// Span of the full socket address.
        address_span: Span,
        /// Span of the host address.
        host_span: Span,
        /// Span of the port.
        port_span: Span,
        /// Underlying [`reqwest::Error`].
        error: reqwest::Error,
    },
    /// Artifact server rejected `app.zip`.
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
    /// Application server failed to get `app.zip`.
    AppZipDownload {
        /// `app.zip` URL file ID.
        app_zip_url_file_id: FileId,
        /// Span of the full URL.
        app_zip_url_span: Span,
        /// Reason provided by the server.
        server_message: Option<String>,
    },
    /// `app.zip` download connection broke.
    AppZipStream {
        /// `app.zip` URL file ID.
        app_zip_url_file_id: FileId,
        /// Span of the full URL.
        app_zip_url_span: Span,
        /// Underlying [`reqwest::Error`].
        error: reqwest::Error,
    },
    /// Web server failed to write `app.zip` to disk.
    AppZipWrite {
        /// `app.zip` path file ID.
        app_zip_path_file_id: FileId,
        /// Span of the app.zip path.
        app_zip_path_span: Span,
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
            Self::AppZipDownload {
                app_zip_url_file_id,
                app_zip_url_span,
                ..
            } => {
                vec![
                    Label::primary(*app_zip_url_file_id, *app_zip_url_span)
                        .with_message("application server failed to get `app.zip`"),
                ]
            }
            Self::AppZipStream {
                app_zip_url_file_id,
                app_zip_url_span,
                ..
            } => {
                vec![
                    Label::primary(*app_zip_url_file_id, *app_zip_url_span)
                        .with_message("`app.zip` download connection failed"),
                ]
            }
            Self::AppZipWrite {
                app_zip_path_file_id,
                app_zip_path_span,
                ..
            } => {
                vec![
                    Label::secondary(*app_zip_path_file_id, *app_zip_path_span)
                        .with_message("failed to write to file"),
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
            Self::AppZipDownload { server_message, .. } => {
                let ensure_hint = String::from("Ensure the file exists on the server.");
                if let Some(server_message) = server_message.as_deref() {
                    let server_message_hint = format!("Message from server:\n{}", server_message);
                    vec![ensure_hint, server_message_hint]
                } else {
                    vec![ensure_hint]
                }
            }
            Self::AppZipStream { error, .. } => {
                vec![format!("Underlying error:\n{}", error)]
            }
            Self::AppZipWrite {
                app_zip_path_file_id,
                app_zip_path_span,
                ..
            } => {
                let app_zip_path = files
                    .source_slice(*app_zip_path_file_id, *app_zip_path_span)
                    .expect("Expected file to exist.");
                vec![format!(
                    "Ensure all parent directories of `{app_zip_path}` are accessible by the current user.",
                    app_zip_path = app_zip_path
                )]
            }
        }
    }
}

impl fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AppZipOpen { .. } => write!(f, "{}", ErrorCode::AppZipOpen.description()),
            Self::ArtifactServerConnect { .. } => {
                write!(f, "{}", ErrorCode::ArtifactServerConnect.description())
            }
            Self::AppZipReject { .. } => write!(f, "{}", ErrorCode::AppZipReject.description()),
            Self::DatabaseCreate { .. } => write!(f, "{}", ErrorCode::DatabaseCreate.description()),
            Self::AppZipDownload { .. } => write!(f, "{}", ErrorCode::AppZipDownload.description()),
            Self::AppZipStream { .. } => write!(f, "{}", ErrorCode::AppZipStream.description()),
            Self::AppZipWrite { .. } => write!(f, "{}", ErrorCode::AppZipStream.description()),
        }
    }
}

impl std::error::Error for ErrorDetail {}

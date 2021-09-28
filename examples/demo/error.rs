use std::{borrow::Cow, fmt, path::Path};

use choochoo::rt_model::{
    error::StationSpecError,
    srcerr::{
        self,
        codespan::{FileId, Files, Span},
        codespan_reporting::diagnostic::Label,
        ErrorCode as _,
    },
};

/// Error codes for simple example.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    /// There is a bug with the station specification.
    StationSpecError,
    /// Failed to build `reqwest::Client`.
    ReqwestClientBuild,
    /// Unable to determine station directory.
    StationDirDiscover,
    /// Unable to determine file name of station file.
    StationFileNameDiscover,
    /// Failed to open `app.zip` to upload.
    AppZipOpen,
    /// Failed to connect to artifact server.
    ArtifactServerConnect,
    /// Artifact server rejected `app.zip`.
    AppZipReject,
    /// Failed to create application database.
    DatabaseCreate,
    /// Failed to open `app.zip` to check state.
    WebServerAppZipOpen,
    /// Failed to read `app.zip` metadata.
    WebServerAppZipMetadata,
    /// Application server failed to get `app.zip`.
    AppZipDownload,
    /// `app.zip` download connection broke.
    AppZipStream,
    /// Web server failed to write `app.zip` to disk.
    AppZipWrite,
    /// Failed to link application to database.
    ApplicationDatabaseLink,
}

impl srcerr::ErrorCode for ErrorCode {
    const ERROR_CODE_MAX: usize = 20;
    const PREFIX: &'static str = "E";

    fn code(self) -> usize {
        match self {
            Self::StationSpecError => 1,
            Self::ReqwestClientBuild => 2,
            Self::StationDirDiscover => 3,
            Self::StationFileNameDiscover => 4,
            Self::AppZipOpen => 5,
            Self::ArtifactServerConnect => 6,
            Self::AppZipReject => 7,
            Self::DatabaseCreate => 8,
            Self::WebServerAppZipOpen => 9,
            Self::WebServerAppZipMetadata => 10,
            Self::AppZipDownload => 11,
            Self::AppZipStream => 12,
            Self::AppZipWrite => 13,
            Self::ApplicationDatabaseLink => 14,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::StationSpecError => "There is a bug with the station specification.",
            Self::ReqwestClientBuild => "Failed to build `reqwest::Client`.",
            Self::StationDirDiscover => "Unable to determine station directory.",
            Self::StationFileNameDiscover => "Unable to determine file name of station file.",
            Self::AppZipOpen => "Failed to open `app.zip` to upload.",
            Self::ArtifactServerConnect => "Failed to connect to artifact server.",
            Self::AppZipReject => "Artifact server rejected `app.zip`.",
            Self::DatabaseCreate => "Failed to create application database.",
            Self::WebServerAppZipOpen => "Failed to open `app.zip` to check state.",
            Self::WebServerAppZipMetadata => "Failed to read `app.zip` metadata.",
            Self::AppZipDownload => "Application server failed to get `app.zip`.",
            Self::AppZipStream => "`app.zip` download connection broke.",
            Self::AppZipWrite => "Web server failed to write `app.zip` to disk.",
            Self::ApplicationDatabaseLink => "Failed to link application to database.",
        }
    }
}

/// Error detail for demo.
#[derive(Debug)]
pub enum ErrorDetail {
    /// There is a bug with the station specification.
    StationSpecError(StationSpecError),
    /// Failed to build `reqwest::Client`.
    ReqwestClientBuild(reqwest::Error),
    /// Unable to determine station directory.
    StationDirDiscover {
        /// File path whose parent should be the station directory.
        station_file_path: &'static Path,
    },
    /// Unable to determine file name of station file.
    StationFileNameDiscover {
        /// File path whose last component should be a file name.
        station_file_path: &'static Path,
    },
    /// Failed to open `app.zip` to upload.
    AppZipOpen {
        /// `app.zip` dir file ID.
        app_zip_dir_file_id: FileId,
        /// `app.zip` path file ID.
        app_zip_path_file_id: FileId,
        /// Span of the app.zip path.
        app_zip_path_span: Span,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Failed to connect to artifact server.
    ArtifactServerConnect {
        /// `app.zip` dir file ID.
        app_zip_dir_file_id: FileId,
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
    /// Failed to open `app.zip` to check state.
    WebServerAppZipOpen {
        /// `app.zip` path file ID.
        app_zip_path_file_id: FileId,
        /// Span of the app.zip path.
        app_zip_path_span: Span,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Failed to read `app.zip` metadata.
    WebServerAppZipMetadata {
        /// `app.zip` path file ID.
        app_zip_path_file_id: FileId,
        /// Span of the app.zip path.
        app_zip_path_span: Span,
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
    /// Failed to link application to database.
    ApplicationDatabaseLink {
        /// Application database link name file ID.
        app_db_link_name_file_id: FileId,
        /// Span of the file name.
        app_db_link_name_span: Span,
        /// Underlying IO error.
        error: std::io::Error,
    },
}

impl<'files> srcerr::ErrorDetail<'files> for ErrorDetail {
    type Files = Files<Cow<'files, str>>;

    fn labels(&self) -> Vec<Label<FileId>> {
        match self {
            Self::StationSpecError(_error) => vec![],
            Self::ReqwestClientBuild(_error) => vec![],
            Self::StationDirDiscover { .. } => vec![],
            Self::StationFileNameDiscover { .. } => vec![],
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
            Self::WebServerAppZipOpen {
                app_zip_path_file_id,
                app_zip_path_span,
                ..
            } => {
                vec![
                    Label::secondary(*app_zip_path_file_id, *app_zip_path_span)
                        .with_message("failed to open file"),
                ]
            }
            Self::WebServerAppZipMetadata {
                app_zip_path_file_id,
                app_zip_path_span,
                ..
            } => {
                vec![
                    Label::secondary(*app_zip_path_file_id, *app_zip_path_span)
                        .with_message("failed to read file metadata"),
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
            Self::ApplicationDatabaseLink {
                app_db_link_name_file_id,
                app_db_link_name_span,
                ..
            } => {
                vec![
                    Label::primary(*app_db_link_name_file_id, *app_db_link_name_span)
                        .with_message("failed to link application to database"),
                ]
            }
        }
    }

    fn notes(&self, files: &Self::Files) -> Vec<String> {
        match self {
            Self::StationSpecError(error) => vec![
                String::from("Make sure the `visit_fn` updates what the `check_fn` is reading."),
                error.to_string(),
            ],
            Self::ReqwestClientBuild(error) => vec![
                String::from("Make sure the `visit_fn` updates what the `check_fn` is reading."),
                error.to_string(),
            ],
            Self::StationDirDiscover { station_file_path } => vec![
                format!(
                    "`{}` is an invalid station file path.",
                    station_file_path.display()
                ),
                String::from(
                    "The station file path should be in a subdirectory for transient artifacts.",
                ),
            ],
            Self::StationFileNameDiscover { station_file_path } => vec![
                format!(
                    "Unable to determine file name for `{}`.",
                    station_file_path.display()
                ),
                String::from("The station file path should not be empty or have trailing slashes."),
            ],
            Self::AppZipOpen {
                app_zip_dir_file_id,
                app_zip_path_file_id,
                app_zip_path_span,
                ..
            } => {
                let app_zip_path = files
                    .source_slice(*app_zip_path_file_id, *app_zip_path_span)
                    .expect("Expected file to exist.");
                let app_zip_dir = files.source(*app_zip_dir_file_id);
                vec![
                    format!(
                        "Try running `ls -l {app_zip_path}` to check file existence and permissions.",
                        app_zip_path = app_zip_path
                    ),
                    format!(
                        "Create the file by running:\n\
                        \n\
                        ```bash\n\
                        mkdir -p {app_zip_dir}\n\
                        for i in {{0..10000}}; do printf \"application contents ${{i}}\\n\"; done | gzip -cf > {app_zip_path}\n\
                        ```",
                        app_zip_dir = app_zip_dir,
                        app_zip_path = app_zip_path
                    ),
                ]
            }
            Self::ArtifactServerConnect {
                app_zip_dir_file_id,
                address_file_id,
                host_span,
                port_span,
                ..
            } => {
                let app_zip_dir = files.source(*app_zip_dir_file_id);
                let host = files
                    .source_slice(*address_file_id, *host_span)
                    .expect("Expected file to exist.");
                let port = files
                    .source_slice(*address_file_id, *port_span)
                    .expect("Expected file to exist.");
                vec![format!(
                    "Try running `cd {app_zip_dir} && simple-http-server --nocache -u --ip {host} --port {port}`.",
                    app_zip_dir = app_zip_dir,
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
                let server_upload_hint = String::from(
                    "Check that the server is accepting uploads. Did you pass `-u` to simple-http-server?",
                );

                if let Some(server_message) = server_message.as_deref() {
                    let server_message_hint = format!("Message from server:\n{}", server_message);
                    vec![zip_valid_hint, server_upload_hint, server_message_hint]
                } else {
                    vec![zip_valid_hint, server_upload_hint]
                }
            }
            Self::DatabaseCreate { .. } => {
                vec![]
            }
            Self::WebServerAppZipOpen {
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
            Self::WebServerAppZipMetadata {
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
            Self::ApplicationDatabaseLink { .. } => {
                vec![]
            }
        }
    }
}

impl fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StationSpecError(error) => error.fmt(f),
            Self::ReqwestClientBuild(error) => error.fmt(f),
            Self::StationDirDiscover { .. } => {
                write!(f, "{}", ErrorCode::StationDirDiscover.description())
            }
            Self::StationFileNameDiscover { .. } => {
                write!(f, "{}", ErrorCode::StationFileNameDiscover.description())
            }
            Self::AppZipOpen { .. } => write!(f, "{}", ErrorCode::AppZipOpen.description()),
            Self::ArtifactServerConnect { .. } => {
                write!(f, "{}", ErrorCode::ArtifactServerConnect.description())
            }
            Self::AppZipReject { .. } => write!(f, "{}", ErrorCode::AppZipReject.description()),
            Self::DatabaseCreate { .. } => write!(f, "{}", ErrorCode::DatabaseCreate.description()),
            Self::WebServerAppZipOpen { .. } => {
                write!(f, "{}", ErrorCode::WebServerAppZipOpen.description())
            }
            Self::WebServerAppZipMetadata { .. } => {
                write!(f, "{}", ErrorCode::WebServerAppZipMetadata.description())
            }
            Self::AppZipDownload { .. } => write!(f, "{}", ErrorCode::AppZipDownload.description()),
            Self::AppZipStream { .. } => write!(f, "{}", ErrorCode::AppZipStream.description()),
            Self::AppZipWrite { .. } => write!(f, "{}", ErrorCode::AppZipStream.description()),
            Self::ApplicationDatabaseLink { .. } => {
                write!(f, "{}", ErrorCode::ApplicationDatabaseLink.description())
            }
        }
    }
}

impl std::error::Error for ErrorDetail {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StationSpecError(error) => Some(error),
            Self::ReqwestClientBuild(error) => Some(error),
            Self::StationDirDiscover { .. } => None,
            Self::StationFileNameDiscover { .. } => None,
            Self::AppZipOpen { error, .. } => Some(error),
            Self::ArtifactServerConnect { error, .. } => Some(error),
            Self::AppZipReject { .. } => None,
            Self::DatabaseCreate { error, .. } => Some(error),
            Self::WebServerAppZipOpen { error, .. } => Some(error),
            Self::WebServerAppZipMetadata { error, .. } => Some(error),
            Self::AppZipDownload { .. } => None,
            Self::AppZipStream { error, .. } => Some(error),
            Self::AppZipWrite { error, .. } => Some(error),
            Self::ApplicationDatabaseLink { error, .. } => Some(error),
        }
    }
}

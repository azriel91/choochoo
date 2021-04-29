use std::{borrow::Cow, path::Path};

use bytes::Bytes;
use choochoo::{
    cfg_model::{CheckStatus, StationFn, StationIdInvalidFmt, StationSpecFns},
    rt_model::Stations,
};
use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use futures::{Stream, StreamExt, TryStreamExt};
use srcerr::{
    codespan::{FileId, Span},
    codespan_reporting::diagnostic::Severity,
};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

use crate::{
    add_station,
    app_zip::{APP_ZIP_APP_SERVER_PARENT, APP_ZIP_APP_SERVER_PATH, APP_ZIP_NAME},
    error::{ErrorCode, ErrorDetail},
    server_params::{ServerParams, SERVER_PARAMS_DEFAULT},
    DemoError, Files,
};

/// Download App
pub struct StationC;

impl StationC {
    /// Returns a station that downloads `app.zip` to a server.
    pub fn build(
        stations: &mut Stations<DemoError>,
    ) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
        let station_spec_fns =
            StationSpecFns::new(Self::visit_fn()).with_check_fn(Self::check_fn());
        add_station(
            stations,
            "c",
            "Download App",
            "Downloads web application onto web server.",
            station_spec_fns,
        )
    }

    fn check_fn() -> StationFn<CheckStatus, DemoError> {
        StationFn::new(move |_station, resources| {
            let client = reqwest::Client::new();
            Box::pin(async move {
                // Short circuit in case the file doesn't exist locally.
                if !Path::new(APP_ZIP_APP_SERVER_PATH).exists() {
                    return Result::<CheckStatus, DemoError>::Ok(CheckStatus::VisitRequired);
                }

                let mut files = resources.borrow_mut::<Files>();

                // TODO: Hash the file and compare with server file hash.
                // Currently we only compare file size
                let local_file_length = {
                    let app_zip = File::open(APP_ZIP_APP_SERVER_PATH)
                        .await
                        .map_err(|error| Self::file_open_error(&mut files, error))?;
                    let metadata = app_zip
                        .metadata()
                        .await
                        .map_err(|error| Self::file_metadata_error(&mut files, error))?;
                    metadata.len()
                };

                let address = Cow::<'_, str>::Owned(SERVER_PARAMS_DEFAULT.address());

                let mut app_zip_url = address.to_string();
                app_zip_url.push('/');
                app_zip_url.push_str(APP_ZIP_NAME);

                let address_file_id = files.add("artifact_server_address", address);
                let address = files.source(address_file_id);

                let response = client.get(&app_zip_url).send().await.map_err(|error| {
                    Self::get_error(&SERVER_PARAMS_DEFAULT, address, address_file_id, error)
                })?;

                let status_code = response.status();
                let check_status = if status_code.is_success() {
                    // We only care about the content length here, so we ignore the response body.
                    if let Some(remote_file_length) = response.content_length() {
                        if local_file_length == remote_file_length {
                            CheckStatus::VisitNotRequired
                        } else {
                            CheckStatus::VisitRequired
                        }
                    } else {
                        // Not sure of file length, so we download it.
                        CheckStatus::VisitRequired
                    }
                } else {
                    // Failed to check. We don't report an error, but maybe we should.
                    CheckStatus::VisitRequired
                };
                Result::<CheckStatus, DemoError>::Ok(check_status)
            })
        })
    }

    fn visit_fn() -> StationFn<(), DemoError> {
        StationFn::new(|_station, resources| {
            let client = reqwest::Client::new();
            Box::pin(async move {
                let mut files = resources.borrow_mut::<Files>();

                let address = Cow::<'_, str>::Owned(SERVER_PARAMS_DEFAULT.address());

                let mut app_zip_url = address.to_string();
                app_zip_url.push('/');
                app_zip_url.push_str(APP_ZIP_NAME);

                let address_file_id = files.add("artifact_server_address", address);
                let address = files.source(address_file_id);

                let response = client.get(&app_zip_url).send().await.map_err(|error| {
                    Self::get_error(&SERVER_PARAMS_DEFAULT, address, address_file_id, error)
                })?;

                let status_code = response.status();
                if status_code.is_success() {
                    Self::app_zip_write(&mut files, app_zip_url, response.bytes_stream()).await?;
                    Result::<(), DemoError>::Ok(())
                } else {
                    let app_zip_url_file_id = files.add(APP_ZIP_NAME, Cow::Owned(app_zip_url));
                    let app_zip_url = files.source(app_zip_url_file_id);
                    let app_zip_url_span = Span::from_str(app_zip_url);
                    let server_message = if let Ok(server_message) = response.text().await {
                        Some(server_message)
                    } else {
                        // Failed to receive response text.
                        // Ignore why the sub-operation failed, but still report the download
                        // reject.
                        None
                    };

                    let code = ErrorCode::AppZipDownload;
                    let detail = ErrorDetail::AppZipDownload {
                        app_zip_url_file_id,
                        app_zip_url_span,
                        server_message,
                    };
                    Err(DemoError::new(code, detail, Severity::Error))
                }
            })
        })
    }

    async fn app_zip_write(
        files: &mut Files,
        app_zip_url: String,
        byte_stream: impl Stream<Item = reqwest::Result<Bytes>>,
    ) -> Result<(), DemoError> {
        let app_zip_url_file_id = files.add(APP_ZIP_NAME, Cow::Owned(app_zip_url.clone()));
        let app_zip_path_file_id = files.add(APP_ZIP_NAME, Cow::Borrowed(APP_ZIP_APP_SERVER_PATH));
        let app_zip_url = files.source(app_zip_url_file_id);
        let app_zip_path = files.source(app_zip_path_file_id);

        tokio::fs::create_dir_all(APP_ZIP_APP_SERVER_PARENT)
            .await
            .map_err(|error| Self::write_error(app_zip_path_file_id, app_zip_path, error))?;
        let app_zip_file = File::create(APP_ZIP_APP_SERVER_PATH)
            .await
            .map_err(|error| Self::write_error(app_zip_path_file_id, app_zip_path, error))?;

        let buffer = BufWriter::new(app_zip_file);
        let mut buffer = byte_stream
            .map(|bytes_result| {
                bytes_result.map_err(|error| {
                    let app_zip_url_span = Span::from_str(app_zip_url);

                    let code = ErrorCode::AppZipStream;
                    let detail = ErrorDetail::AppZipStream {
                        app_zip_url_file_id,
                        app_zip_url_span,
                        error,
                    };
                    DemoError::new(code, detail, Severity::Error)
                })
            })
            .try_fold(buffer, |mut buffer, bytes| async move {
                buffer.write_all(&bytes).await.map_err(|error| {
                    Self::write_error(app_zip_path_file_id, app_zip_path, error)
                })?;

                Ok(buffer)
            })
            .await?;
        buffer
            .flush()
            .await
            .map_err(|error| Self::write_error(app_zip_path_file_id, app_zip_path, error))?;

        Ok(())
    }

    fn file_open_error(files: &mut Files, error: std::io::Error) -> DemoError {
        let app_zip_path_file_id = files.add(APP_ZIP_NAME, Cow::Borrowed(APP_ZIP_APP_SERVER_PATH));
        let app_zip_path = files.source(app_zip_path_file_id);
        let app_zip_path_span = Span::from_str(app_zip_path);

        let code = ErrorCode::WebServerAppZipOpen;
        let detail = ErrorDetail::WebServerAppZipOpen {
            app_zip_path_file_id,
            app_zip_path_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }

    fn file_metadata_error(files: &mut Files, error: std::io::Error) -> DemoError {
        let app_zip_path_file_id = files.add(APP_ZIP_NAME, Cow::Borrowed(APP_ZIP_APP_SERVER_PATH));
        let app_zip_path = files.source(app_zip_path_file_id);
        let app_zip_path_span = Span::from_str(app_zip_path);

        let code = ErrorCode::WebServerAppZipMetadata;
        let detail = ErrorDetail::WebServerAppZipMetadata {
            app_zip_path_file_id,
            app_zip_path_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }

    fn write_error(
        app_zip_path_file_id: FileId,
        app_zip_path: &str,
        error: std::io::Error,
    ) -> DemoError {
        let app_zip_path_span = Span::from_str(app_zip_path);

        let code = ErrorCode::AppZipWrite;
        let detail = ErrorDetail::AppZipWrite {
            app_zip_path_file_id,
            app_zip_path_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }

    fn get_error(
        server_params: &ServerParams,
        address: &str,
        address_file_id: FileId,
        error: reqwest::Error,
    ) -> DemoError {
        let ServerParams {
            protocol,
            host,
            port,
        } = server_params;

        let address_span = Span::new(protocol.len() as u32, address.len() as u32);
        let host_start = protocol.len();
        let host_end = host_start + host.len();
        let port_start = host_end + 1;
        let port_end = port_start + port.len();
        let host_span = Span::new(host_start as u32, host_end as u32);
        let port_span = Span::new(port_start as u32, port_end as u32);

        let code = ErrorCode::ArtifactServerConnect;
        let detail = ErrorDetail::ArtifactServerConnect {
            address_file_id,
            address_span,
            host_span,
            port_span,
            error,
        };
        DemoError::new(code, detail, Severity::Error)
    }
}

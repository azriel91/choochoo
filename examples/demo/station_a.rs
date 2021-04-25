use std::borrow::Cow;

use choochoo::{
    cfg_model::{StationFn, StationIdInvalidFmt},
    rt_model::Stations,
};
use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use srcerr::{
    codespan::{FileId, Span},
    codespan_reporting::diagnostic::Severity,
};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{
    add_station,
    error::{ErrorCode, ErrorDetail},
    DemoError, Files,
};

pub struct StationA;

impl StationA {
    /// Returns a station that uploads "app.zip" to a server.
    pub fn build(
        stations: &mut Stations<DemoError>,
    ) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
        let visit_fn = StationFn::new(|_station, resources| {
            let client = reqwest::Client::new();
            Box::pin(async move {
                let mut files = resources.borrow_mut::<Files>();

                let app_zip_byte_stream = Self::app_zip_read(&mut files).await?;

                let address = Cow::Owned(SERVER_PARAMS.address());
                let address_file_id = files.add("artifact_server_address", address);
                let address = files.source(address_file_id);

                let response = client
                    .post(&**address)
                    .body(reqwest::Body::wrap_stream(app_zip_byte_stream))
                    .send()
                    .await
                    .map_err(|error| {
                        Self::post_error(&SERVER_PARAMS, address, address_file_id, error)
                    })?;

                let status_code = response.status();
                if status_code.is_success() {
                    Result::<(), DemoError>::Ok(())
                } else {
                    let address_span = Span::from_str(address);
                    let app_zip_path_file_id = files.add("app.zip", Cow::Borrowed(APP_ZIP_PATH));
                    let app_zip_path = files.source(app_zip_path_file_id);
                    let app_zip_path_span = Span::from_str(app_zip_path);
                    let server_message = if let Ok(server_message) = response.text().await {
                        Some(server_message)
                    } else {
                        // Failed to receive response text.
                        // Ignore why that sub-operation failed, but we still
                        // report the upload reject.
                        None
                    };

                    let code = ErrorCode::AppZipReject;
                    let detail = ErrorDetail::AppZipReject {
                        app_zip_path_file_id,
                        app_zip_path_span,
                        address_file_id,
                        address_span,
                        server_message,
                    };
                    Err(DemoError::new(code, detail, Severity::Error))
                }
            })
        });
        add_station(
            stations,
            "a",
            "Upload App",
            "Uploads web application to artifact server.",
            visit_fn,
        )
    }

    async fn app_zip_read(files: &mut Files) -> Result<FramedRead<File, BytesCodec>, DemoError> {
        let app_zip_read = File::open(APP_ZIP_PATH).await.map_err(|error| {
            let app_zip_path_file_id = files.add("app.zip", Cow::Borrowed(APP_ZIP_PATH));
            let app_zip_path = files.source(app_zip_path_file_id);
            let app_zip_path_span = Span::from_str(app_zip_path);

            let code = ErrorCode::AppZipOpen;
            let detail = ErrorDetail::AppZipOpen {
                app_zip_path_file_id,
                app_zip_path_span,
                error,
            };

            DemoError::new(code, detail, Severity::Error)
        })?;
        Ok(FramedRead::new(app_zip_read, BytesCodec::new()))
    }

    fn post_error(
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

const APP_ZIP_PATH: &'static str = "/tmp/build_agent/app.zip";

pub struct ServerParams {
    protocol: &'static str,
    host: &'static str,
    port: &'static str,
}

const SERVER_PARAMS: ServerParams = ServerParams {
    protocol: "http://",
    host: "127.0.0.1",
    port: "8000",
};

impl ServerParams {
    pub fn address(&self) -> String {
        format!(
            "{protocol}{host}:{port}",
            protocol = self.protocol,
            host = self.host,
            port = self.port
        )
    }
}

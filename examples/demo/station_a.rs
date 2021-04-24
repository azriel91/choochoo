use std::borrow::Cow;

use choochoo::{
    cfg_model::{StationFn, StationIdInvalidFmt},
    rt_model::Stations,
};
use codespan::{FileId, Span};
use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use srcerr::codespan_reporting::diagnostic::Severity;

use crate::{
    add_station,
    error::{ErrorCode, ErrorDetail},
    ExampleError, Files,
};

pub struct StationA;

impl StationA {
    /// Returns a station that uploads "app.zip" to a server.
    pub fn build(
        stations: &mut Stations<ExampleError<'_>>,
    ) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
        let visit_fn = StationFn::new(|_station, resources| {
            Box::pin(async move {
                let mut files = resources.borrow_mut::<Files>();

                let address = Cow::Owned(SERVER_PARAMS.address());
                let address_file_id = files.add("artifact_server_address", address);
                let address = files.source(address_file_id);

                reqwest::get(&**address).await.map_err(|error| {
                    Self::error(&SERVER_PARAMS, address, address_file_id, error)
                })?;

                Result::<(), ExampleError<'_>>::Ok(())
            })
        });
        add_station(
            stations,
            "a",
            "Upload App",
            "Uploads web application to S3.",
            visit_fn,
        )
    }

    fn error(
        server_params: &ServerParams,
        address: &str,
        address_file_id: FileId,
        error: reqwest::Error,
    ) -> ExampleError<'static> {
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
        ExampleError::new(code, detail, Severity::Error)
    }
}

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

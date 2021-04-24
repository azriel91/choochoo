use std::borrow::Cow;

use choochoo::{
    cfg_model::{StationFn, StationIdInvalidFmt},
    rt_model::Stations,
};
use codespan::Span;
use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use srcerr::{codespan_reporting::diagnostic::Severity, SourceError};

use crate::{
    add_station,
    error::{ErrorCode, ErrorDetail},
    ExampleError, Files,
};

pub fn station_a(
    stations: &mut Stations<ExampleError<'_>>,
) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
    let visit_fn = StationFn::new(|_station, resources| {
        Box::pin(async move {
            let protocol = "http://";
            let host = "127.0.0.1";
            let port = "8000";
            let address = Cow::Owned(format!(
                "{protocol}{host}:{port}",
                protocol = protocol,
                host = host,
                port = port
            ));
            let mut files = resources.borrow_mut::<Files>();
            let address_file_id = files.add("artifact_server_address", address);
            let address = files.source(address_file_id);

            let address_span = Span::new(protocol.len() as u32, address.len() as u32);

            let host_start = protocol.len();
            let host_end = host_start + host.len();
            let port_start = host_end + 1;
            let port_end = port_start + port.len();
            let host_span = Span::new(host_start as u32, host_end as u32);
            let port_span = Span::new(port_start as u32, port_end as u32);

            reqwest::get(&**address).await.map_err(|error| {
                let code = ErrorCode::ArtifactServerConnect;
                let detail = ErrorDetail::ArtifactServerConnect {
                    address_file_id,
                    address_span,
                    host_span,
                    port_span,
                    error,
                };
                SourceError::new(code, detail, Severity::Error)
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

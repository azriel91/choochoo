use choochoo::cfg_model::srcerr::{
    codespan::{FileId, Span},
    codespan_reporting::diagnostic::Severity,
};

use crate::{
    error::{ErrorCode, ErrorDetail},
    server_params::ServerParams,
    DemoError,
};

pub(crate) struct StationAErrors;

impl StationAErrors {
    pub(crate) fn connect_error(
        server_params: &ServerParams,
        artifact_server_dir_file_id: FileId,
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
            artifact_server_dir_file_id,
            address_file_id,
            address_span,
            host_span,
            port_span,
            error,
        };
        DemoError::new(code, detail, Severity::Error)
    }
}

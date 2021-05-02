use std::path::Path;

use crate::station_sleep::StationSleep;
use choochoo::{
    cfg_model::{StationId, StationIdInvalidFmt},
    rt_model::Station,
};

use srcerr::{
    codespan::{FileId, Span},
    codespan_reporting::diagnostic::Severity,
};

use crate::{DemoError, ErrorCode, ErrorDetail};

/// Create DB
pub struct StationB;

impl StationB {
    /// Creates the database for the web application.
    pub fn build() -> Result<Station<DemoError>, StationIdInvalidFmt<'static>> {
        let station_id = StationId::new("b")?;
        let station_name = String::from("Create DB");
        let station_description = String::from("Creates the database for the web application.");
        let station = StationSleep::new(
            station_id,
            station_name,
            station_description,
            &Path::new("/tmp/choochoo/demo/station_b/create_db"),
            Self::db_error,
        );
        Ok(station)
    }

    fn db_error(db_name_file_id: FileId, db_name_span: Span, error: std::io::Error) -> DemoError {
        let code = ErrorCode::DatabaseCreate;
        let detail = ErrorDetail::DatabaseCreate {
            db_name_file_id,
            db_name_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }
}

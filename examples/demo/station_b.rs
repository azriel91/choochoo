use std::path::Path;

use choochoo::{
    cfg_model::{StationId, StationIdInvalidFmt},
    rt_model::{
        srcerr::{
            codespan::{FileId, Span},
            codespan_reporting::diagnostic::Severity,
        },
        StationProgresses, StationRtId, StationSpecs,
    },
};

use crate::{station_sleep::StationSleep, DemoError, ErrorCode, ErrorDetail};

/// Create DB
pub struct StationB;

impl StationB {
    /// Creates the database for the web application.
    pub fn build(
        station_specs: &mut StationSpecs<DemoError>,
        station_progresses: &mut StationProgresses<DemoError>,
    ) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
        let station_id = StationId::new("b")?;
        let station_name = String::from("Create DB");
        let station_description = String::from("Creates the database for the web application.");
        let station_rt_id = StationSleep::new(
            station_specs,
            station_progresses,
            station_id,
            station_name,
            station_description,
            &Path::new("/tmp/choochoo/demo/station_b/create_db"),
            Self::db_error,
        );
        Ok(station_rt_id)
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

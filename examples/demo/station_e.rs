use std::path::Path;

use choochoo::{
    cfg_model::{StationId, StationIdInvalidFmt},
    rt_model::{StationProgresses, StationRtId, Stations},
};
use srcerr::{
    codespan::{FileId, Span},
    codespan_reporting::diagnostic::Severity,
};

use crate::{station_sleep::StationSleep, DemoError, ErrorCode, ErrorDetail};

/// Run App
pub struct StationE;

impl StationE {
    /// Starts the web application service.
    pub fn build(
        stations: &mut Stations<DemoError>,
        station_progresses: &mut StationProgresses<DemoError>,
    ) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
        let station_id = StationId::new("e")?;
        let station_name = String::from("Run App");
        let station_description = String::from("Starts the web application service.");
        let station_rt_id = StationSleep::new(
            stations,
            station_progresses,
            station_id,
            station_name,
            station_description,
            &Path::new("/tmp/choochoo/demo/station_e/run_app"),
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

use std::path::Path;

use choochoo::cfg_model::{
    srcerr::{
        codespan::{FileId, Span},
        codespan_reporting::diagnostic::Severity,
    },
    StationId, StationIdInvalidFmt, StationSpec,
};

use crate::{station_sleep::StationSleep, DemoError, ErrorCode, ErrorDetail};

/// Link App to DB
pub struct StationD;

impl StationD {
    /// Links the web application to the database.
    pub fn build() -> Result<StationSpec<DemoError>, StationIdInvalidFmt<'static>> {
        let station_id = StationId::new("d")?;
        let station_name = String::from("Link App to DB");
        let station_description = String::from("Links the web application to the database.");
        let station_rt_id = StationSleep::new(
            station_id,
            station_name,
            station_description,
            &Path::new("/tmp/choochoo/demo/station_d/link_app_to_db"),
            Self::db_error,
        );
        Ok(station_rt_id)
    }

    fn db_error(
        app_db_link_name_file_id: FileId,
        app_db_link_name_span: Span,
        error: std::io::Error,
    ) -> DemoError {
        let code = ErrorCode::ApplicationDatabaseLink;
        let detail = ErrorDetail::ApplicationDatabaseLink {
            app_db_link_name_file_id,
            app_db_link_name_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }
}

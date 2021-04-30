use std::{borrow::Cow, path::Path};

use choochoo::{
    cfg_model::{
        CheckStatus, StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns,
    },
    rt_model::{Files, Station, VisitStatus},
};
use futures::{stream, stream::StreamExt};
use srcerr::{codespan::Span, codespan_reporting::diagnostic::Severity};
use tokio::time::Duration;

use crate::{DemoError, ErrorCode, ErrorDetail};

/// Link App to DB
pub struct StationD;

impl StationD {
    /// Links the web application to the database.
    pub fn build() -> Result<Station<DemoError>, StationIdInvalidFmt<'static>> {
        let station_spec_fns =
            StationSpecFns::new(Self::visit_fn()).with_check_fn(Self::check_fn());
        let station_id = StationId::new("d")?;
        let station_name = String::from("Link App to DB");
        let station_description = String::from("Links the web application to the database.");
        let station_spec = StationSpec::new(
            station_id,
            station_name,
            station_description,
            station_spec_fns,
        );
        let station = Station::new(station_spec, VisitStatus::NotReady);
        Ok(station)
    }

    fn check_fn() -> StationFn<CheckStatus, DemoError> {
        StationFn::new(move |station, _resources| {
            Box::pin(async move {
                station.progress_bar.reset();
                station.progress_bar.tick();
                let check_status = if Path::new(LINK_APP_TO_DB_PATH).exists() {
                    CheckStatus::VisitNotRequired
                } else {
                    CheckStatus::VisitRequired
                };
                Result::<CheckStatus, DemoError>::Ok(check_status)
            })
        })
    }

    fn visit_fn() -> StationFn<(), DemoError> {
        StationFn::new(move |station, resources| {
            Box::pin(async move {
                let mut files = resources.borrow_mut::<Files>();

                // Sleep to simulate linking app to database.
                station.progress_bar.reset();
                stream::iter(0..100)
                    .for_each(|_| async {
                        station.progress_bar.inc(1);
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    })
                    .await;

                tokio::fs::create_dir_all(LINK_APP_TO_DB_PARENT)
                    .await
                    .map_err(|error| Self::db_error(&mut files, error))?;
                tokio::fs::write(LINK_APP_TO_DB_PATH, b"Application linked to DB!\n")
                    .await
                    .map_err(|error| Self::db_error(&mut files, error))?;

                Result::<(), DemoError>::Ok(())
            })
        })
    }

    fn db_error(files: &mut Files, error: std::io::Error) -> DemoError {
        let app_db_link_name_file_id =
            files.add(LINK_APP_TO_DB_NAME, Cow::Borrowed(LINK_APP_TO_DB_PATH));
        let app_db_link_name = files.source(app_db_link_name_file_id);
        let app_db_link_name_span = Span::from_str(app_db_link_name);

        let code = ErrorCode::ApplicationDatabaseLink;
        let detail = ErrorDetail::ApplicationDatabaseLink {
            app_db_link_name_file_id,
            app_db_link_name_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }
}

const LINK_APP_TO_DB_NAME: &'static str = "link_app_to_db";
const LINK_APP_TO_DB_PARENT: &'static str = "/tmp/choochoo/demo/station_d";
const LINK_APP_TO_DB_PATH: &'static str = "/tmp/choochoo/demo/station_d/link_app_to_db";

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

/// Run App
pub struct StationE;

impl StationE {
    /// Starts the web application service.
    pub fn build() -> Result<Station<DemoError>, StationIdInvalidFmt<'static>> {
        let station_spec_fns =
            StationSpecFns::new(Self::visit_fn()).with_check_fn(Self::check_fn());
        let station_id = StationId::new("e")?;
        let station_name = String::from("Run App");
        let station_description = String::from("Starts the web application service.");
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
                let check_status = if Path::new(RUN_APP_PATH).exists() {
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

                // Sleep to simulate starting up the application.
                station.progress_bar.reset();
                stream::iter(0..100)
                    .for_each(|_| async {
                        station.progress_bar.inc(1);
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    })
                    .await;

                tokio::fs::create_dir_all(RUN_APP_PARENT)
                    .await
                    .map_err(|error| Self::db_error(&mut files, error))?;
                tokio::fs::write(RUN_APP_PATH, b"Application started!\n")
                    .await
                    .map_err(|error| Self::db_error(&mut files, error))?;

                Result::<(), DemoError>::Ok(())
            })
        })
    }

    fn db_error(files: &mut Files, error: std::io::Error) -> DemoError {
        let db_name_file_id = files.add(RUN_APP_NAME, Cow::Borrowed(RUN_APP_PATH));
        let db_name = files.source(db_name_file_id);
        let db_name_span = Span::from_str(db_name);

        let code = ErrorCode::DatabaseCreate;
        let detail = ErrorDetail::DatabaseCreate {
            db_name_file_id,
            db_name_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }
}

const RUN_APP_NAME: &'static str = "run_app";
const RUN_APP_PARENT: &'static str = "/tmp/choochoo/demo/station_e";
const RUN_APP_PATH: &'static str = "/tmp/choochoo/demo/station_e/run_app";

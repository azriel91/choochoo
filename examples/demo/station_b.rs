use std::borrow::Cow;

use choochoo::{
    cfg_model::{StationFn, StationIdInvalidFmt},
    rt_model::{Files, Stations},
};
use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use srcerr::{codespan::Span, codespan_reporting::diagnostic::Severity};
use tokio::time::Duration;

use crate::{add_station, DemoError, ErrorCode, ErrorDetail};

pub struct StationB;

impl StationB {
    /// Returns a station that uploads `app.zip` to a server.
    pub fn build(
        stations: &mut Stations<DemoError>,
    ) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
        let visit_fn = StationFn::new(move |_station, resources| {
            Box::pin(async move {
                let mut files = resources.borrow_mut::<Files>();

                // Sleep to simulate creating a database.
                tokio::time::sleep(Duration::from_secs(1)).await;

                tokio::fs::create_dir_all(CREATE_DB_PARENT)
                    .await
                    .map_err(|error| Self::db_error(&mut files, error))?;
                tokio::fs::write(CREATE_DB_PATH, b"Database created!\n")
                    .await
                    .map_err(|error| Self::db_error(&mut files, error))?;

                Result::<(), DemoError>::Ok(())
            })
        });
        add_station(
            stations,
            "b",
            "Create DB",
            "Creates the database for the web application.",
            visit_fn,
        )
    }

    fn db_error(files: &mut Files, error: std::io::Error) -> DemoError {
        let db_name_file_id = files.add(CREATE_DB_NAME, Cow::Borrowed(CREATE_DB_PATH));
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

const CREATE_DB_NAME: &'static str = "create_db";
const CREATE_DB_PARENT: &'static str = "/tmp/choochoo/demo/station_b";
const CREATE_DB_PATH: &'static str = "/tmp/choochoo/demo/station_b/create_db";

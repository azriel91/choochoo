use std::path::Path;

use choochoo::{
    cfg_model::{
        CheckStatus, StationFn, StationId, StationProgress, StationSpec, StationSpecFns,
        StationSpecs, VisitStatus,
    },
    rt_model::{
        srcerr::{
            codespan::{FileId, Span},
            codespan_reporting::diagnostic::Severity,
        },
        Files, RwFiles, StationProgresses, StationRtId,
    },
};
use futures::{stream, stream::StreamExt};
use tokio::time::Duration;

use crate::{DemoError, ErrorCode, ErrorDetail};

const PROGRESS_LENGTH: u64 = 100;

/// Sleeps to simulate a process
pub struct StationSleep;

impl StationSleep {
    /// Sleeps to simulate a process
    pub fn new(
        station_specs: &mut StationSpecs<DemoError>,
        station_progresses: &mut StationProgresses,
        station_id: StationId,
        station_name: String,
        station_description: String,
        station_file_path: &'static Path,
        error_fn: fn(FileId, Span, std::io::Error) -> DemoError,
    ) -> StationRtId {
        let station_spec_fns = StationSpecFns::new(Self::visit_fn(station_file_path, error_fn))
            .with_check_fn(Self::check_fn(station_file_path));
        let station_spec = StationSpec::new(
            station_id,
            station_name,
            station_description,
            station_spec_fns,
        );
        let station_progress = StationProgress::new(&station_spec, VisitStatus::NotReady);
        station_progress.progress_bar.set_length(PROGRESS_LENGTH);

        let station_rt_id = station_specs.add_node(station_spec);
        station_progresses.insert(station_rt_id, station_progress);

        station_rt_id
    }

    fn check_fn(station_file_path: &'static Path) -> StationFn<CheckStatus, DemoError> {
        StationFn::new(move |_station, _resources| {
            Box::pin(async move {
                let check_status = if station_file_path.exists() {
                    CheckStatus::VisitNotRequired
                } else {
                    CheckStatus::VisitRequired
                };
                Result::<CheckStatus, DemoError>::Ok(check_status)
            })
        })
    }

    fn visit_fn(
        station_file_path: &'static Path,
        error_fn: fn(FileId, Span, std::io::Error) -> DemoError,
    ) -> StationFn<(), DemoError> {
        StationFn::new(move |station_progress, resources| {
            Box::pin(async move {
                // Sleep to simulate starting up the application.
                station_progress.progress_bar.reset();
                stream::iter(0..PROGRESS_LENGTH)
                    .for_each(|_| async {
                        station_progress.progress_bar.inc(1);
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    })
                    .await;

                let station_dir = station_file_path.parent().ok_or_else(|| {
                    let code = ErrorCode::StationDirDiscover;
                    let detail = ErrorDetail::StationDirDiscover { station_file_path };
                    DemoError::new(code, detail, Severity::Bug)
                })?;
                let files = resources.borrow::<RwFiles>();
                let mut files = files.write().await;
                tokio::fs::create_dir_all(station_dir)
                    .await
                    .map_err(|error| {
                        match Self::write_error(&mut files, station_file_path, error, error_fn) {
                            Ok(e) | Err(e) => e,
                        }
                    })?;
                tokio::fs::write(station_file_path, b"Station visited!\n")
                    .await
                    .map_err(|error| {
                        match Self::write_error(&mut files, station_file_path, error, error_fn) {
                            Ok(e) | Err(e) => e,
                        }
                    })?;

                Result::<(), DemoError>::Ok(())
            })
        })
    }

    fn write_error(
        files: &mut Files,
        station_file_path: &'static Path,
        error: std::io::Error,
        error_fn: fn(FileId, Span, std::io::Error) -> DemoError,
    ) -> Result<DemoError, DemoError> {
        let station_file_name = station_file_path.file_name().ok_or_else(|| {
            let code = ErrorCode::StationFileNameDiscover;
            let detail = ErrorDetail::StationFileNameDiscover { station_file_path };
            DemoError::new(code, detail, Severity::Bug)
        })?;
        let station_file_name_file_id =
            files.add(station_file_name, station_file_path.to_string_lossy());
        let station_file_name = files.source(station_file_name_file_id);
        let station_file_name_span = Span::from_str(station_file_name);

        Ok(error_fn(
            station_file_name_file_id,
            station_file_name_span,
            error,
        ))
    }
}

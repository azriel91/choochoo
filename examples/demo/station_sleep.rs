use std::path::Path;

use choochoo::{
    cfg_model::{
        rt::{CheckStatus, ProgressLimit, ResIds, StationMutRef},
        srcerr::{
            codespan::{FileId, Span},
            codespan_reporting::diagnostic::Severity,
        },
        CreateFns, SetupFn, StationFn, StationId, StationOp, StationSpec,
    },
    resource::{Files, FilesRw},
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
        station_id: StationId,
        station_name: String,
        station_description: String,
        station_file_path: &'static Path,
        error_fn: fn(FileId, Span, std::io::Error) -> DemoError,
    ) -> StationSpec<DemoError> {
        let create_fns =
            CreateFns::new(Self::setup_fn(), Self::work_fn(station_file_path, error_fn))
                .with_check_fn(Self::check_fn(station_file_path));
        let station_op = StationOp::new(create_fns, None);
        StationSpec::new(station_id, station_name, station_description, station_op)
    }

    fn setup_fn() -> SetupFn<DemoError> {
        SetupFn::new(move |_station, _train_resources| {
            Box::pin(async move { Ok(ProgressLimit::Steps(PROGRESS_LENGTH)) })
        })
    }

    fn check_fn(station_file_path: &'static Path) -> StationFn<CheckStatus, DemoError, DemoError> {
        StationFn::new0(move |_station| {
            Box::pin(async move {
                let check_status = if station_file_path.exists() {
                    CheckStatus::WorkNotRequired
                } else {
                    CheckStatus::WorkRequired
                };
                Result::<CheckStatus, DemoError>::Ok(check_status)
            })
        })
    }

    fn work_fn(
        station_file_path: &'static Path,
        error_fn: fn(FileId, Span, std::io::Error) -> DemoError,
    ) -> StationFn<ResIds, (ResIds, DemoError), DemoError> {
        StationFn::new1(
            move |station: &mut StationMutRef<'_, DemoError>, files: &FilesRw| {
                Box::pin(async move {
                    // Sleep to simulate starting up the application.
                    station.progress.progress_bar().reset();
                    stream::iter(0..PROGRESS_LENGTH)
                        .for_each(|_| async {
                            station.progress.progress_bar().inc(1);
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        })
                        .await;
                    let res_ids = ResIds::new();

                    let station_dir = station_file_path
                        .parent()
                        .ok_or_else(|| {
                            let code = ErrorCode::StationDirDiscover;
                            let detail = ErrorDetail::StationDirDiscover { station_file_path };
                            DemoError::new(code, detail, Severity::Bug)
                        })
                        .map_err(|e| (res_ids.clone(), e))?;
                    let mut files = files.write().await;
                    tokio::fs::create_dir_all(station_dir)
                        .await
                        .map_err(|error| {
                            match Self::write_error(&mut files, station_file_path, error, error_fn)
                            {
                                Ok(e) | Err(e) => e,
                            }
                        })
                        .map_err(|e| (res_ids.clone(), e))?;
                    tokio::fs::write(station_file_path, b"Station visited!\n")
                        .await
                        .map_err(|error| {
                            match Self::write_error(&mut files, station_file_path, error, error_fn)
                            {
                                Ok(e) | Err(e) => e,
                            }
                        })
                        .map_err(|e| (res_ids.clone(), e))?;

                    Ok(res_ids)
                })
            },
        )
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

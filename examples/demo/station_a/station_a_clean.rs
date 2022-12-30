use std::borrow::Cow;

use choochoo::{
    cfg_model::{
        rt::{CheckStatus, StationMutRef},
        srcerr::{codespan::Span, codespan_reporting::diagnostic::Severity},
        CleanFns, SetupFn, StationFn,
    },
    resource::FilesRw,
};
use futures::future::{FutureExt, LocalBoxFuture};
use reqwest::StatusCode;

use crate::{
    app_zip::APP_ZIP_NAME, artifact_server_dir::ArtifactServerDir,
    server_params::SERVER_PARAMS_DEFAULT, station_a::StationAErrors, DemoError, ErrorCode,
    ErrorDetail,
};

pub(crate) struct StationAClean;

impl StationAClean {
    /// Returns the create functions for Station A.
    pub(crate) fn build() -> CleanFns<DemoError> {
        CleanFns::new(Self::setup_fn(), StationFn::new(Self::work_fn))
            .with_check_fn(StationFn::new(Self::check_fn))
    }

    pub(crate) fn setup_fn() -> SetupFn<DemoError> {
        SetupFn::new(|_station, _train_resources| {
            todo!("Register resource id types with type registry.")
            // async move { Ok(ProgressLimit::Steps(1)) }.boxed_local()
        })
    }

    pub(crate) fn check_fn<'f>(
        _station: &'f mut StationMutRef<'_, DemoError>,
        files: &'f FilesRw,
        artifact_server_dir: &'f ArtifactServerDir,
    ) -> LocalBoxFuture<'f, Result<CheckStatus, DemoError>> {
        let client = reqwest::Client::new();
        Box::pin(async move {
            let mut files = files.write().await;

            let address = Cow::<'_, str>::Owned(SERVER_PARAMS_DEFAULT.address());

            let mut app_zip_url = address.to_string();
            app_zip_url.push('/');
            app_zip_url.push_str(APP_ZIP_NAME);

            let address_file_id = files.add("artifact_server_address", address);

            let response = client.get(&app_zip_url).send().await.map_err(|error| {
                let artifact_server_dir_file_id = files.add(
                    artifact_server_dir,
                    Cow::Owned(artifact_server_dir.to_string_lossy().into_owned()),
                );
                let address = files.source(address_file_id);
                StationAErrors::connect_error(
                    &SERVER_PARAMS_DEFAULT,
                    artifact_server_dir_file_id,
                    address,
                    address_file_id,
                    error,
                )
            })?;

            let status_code = response.status();
            let check_status = if status_code == StatusCode::NOT_FOUND {
                CheckStatus::WorkNotRequired
            } else {
                // We only care about the existence of the file, so we ignore the response body.
                // This path is also taken if the server returns 5xx, in which case we don't
                // report an error, but maybe we should.
                CheckStatus::WorkRequired
            };

            Ok(check_status)
        })
    }

    pub(crate) fn work_fn<'f>(
        _station: &'f mut StationMutRef<'_, DemoError>,
        files: &'f FilesRw,
        artifact_server_dir: &'f ArtifactServerDir,
    ) -> LocalBoxFuture<'f, Result<(), DemoError>> {
        async move {
            let app_zip_file_path = artifact_server_dir.join(APP_ZIP_NAME);
            let remove_result = tokio::fs::remove_file(&app_zip_file_path).await;

            match remove_result {
                Ok(()) => Ok(()),
                Err(error) => {
                    let mut files = files.write().await;
                    let app_zip_file_path_string = format!("{}", app_zip_file_path.display());
                    let app_zip_path_span = app_zip_file_path
                        .file_name()
                        .and_then(|file_name| file_name.to_str())
                        .and_then(|file_name| {
                            let index = app_zip_file_path_string.rfind(file_name);
                            index.map(|index| (index, file_name))
                        })
                        .map(|(index, file_name)| Span::new(index as u32, file_name.len() as u32))
                        .unwrap_or_else(|| Span::new(0, app_zip_file_path_string.len() as u32));

                    let app_zip_path_file_id =
                        files.add(&app_zip_file_path, Cow::Owned(app_zip_file_path_string));
                    let demo_error = DemoError::new(
                        ErrorCode::CleanArtifactServerAppZip,
                        ErrorDetail::CleanArtifactServerAppZip {
                            app_zip_path_file_id,
                            app_zip_path_span,
                            error,
                        },
                        Severity::Error,
                    );

                    Err(demo_error)
                }
            }
        }
        .boxed_local()
    }
}

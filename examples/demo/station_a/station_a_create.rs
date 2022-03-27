use std::{borrow::Cow, path::Path};

use choochoo::{
    cfg_model::{
        rt::{CheckStatus, ProgressLimit, ResIdLogical, ResIds, StationMut, StationMutRef},
        srcerr::{codespan::Span, codespan_reporting::diagnostic::Severity},
        CreateFns, SetupFn, StationFn,
    },
    resource::{Files, FilesRw, ProfileDir},
};
use futures::future::LocalBoxFuture;
use reqwest::{
    multipart::{Form, Part},
    redirect::Policy,
};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{
    app_zip::{AppZipFileLength, APP_ZIP_NAME},
    artifact_server_dir::ArtifactServerDir,
    error::{ErrorCode, ErrorDetail},
    server_params::SERVER_PARAMS_DEFAULT,
    station_a::StationAErrors,
    DemoError,
};

pub(crate) struct StationACreate;

impl StationACreate {
    /// Returns the create functions for Station A.
    pub(crate) fn build() -> CreateFns<DemoError> {
        CreateFns::new(Self::setup_fn(), StationFn::new(Self::work_fn))
            .with_check_fn(StationFn::new(Self::check_fn))
    }

    fn setup_fn() -> SetupFn<DemoError> {
        SetupFn::new(|station, train_resources| {
            Box::pin(async move {
                let local_file_length = {
                    let files = train_resources.borrow::<FilesRw>();
                    let mut files = files.write().await;

                    let app_zip_build_agent_path = station.dir.join(APP_ZIP_NAME);
                    let app_zip = File::open(&app_zip_build_agent_path)
                        .await
                        .map_err(|error| {
                            Self::file_open_error(
                                station,
                                &mut files,
                                &app_zip_build_agent_path,
                                error,
                            )
                        })?;
                    let metadata = app_zip.metadata().await.map_err(|error| {
                        Self::file_metadata_error(&mut files, &app_zip_build_agent_path, error)
                    })?;
                    metadata.len()
                };

                train_resources.insert(AppZipFileLength(local_file_length));

                let artifact_server_dir = {
                    let profile_dir = train_resources.borrow::<ProfileDir>();
                    ArtifactServerDir::new(profile_dir.to_path_buf())
                };
                train_resources.insert(artifact_server_dir);

                Ok(ProgressLimit::Bytes(local_file_length))
            })
        })
    }

    fn check_fn<'f>(
        _station: &'f mut StationMutRef<'_, DemoError>,
        files: &'f FilesRw,
        artifact_server_dir: &'f ArtifactServerDir,
        local_file_length: &'f AppZipFileLength,
    ) -> LocalBoxFuture<'f, Result<CheckStatus, DemoError>> {
        let client = reqwest::Client::new();
        Box::pin(async move {
            let mut files = files.write().await;

            // TODO: Hash the file and compare with server file hash.
            // Currently we only compare file size
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
            let check_status = if status_code.is_success() {
                // We only care about the content length here, so we ignore the response
                // body.
                if let Some(remote_file_length) = response.content_length() {
                    if local_file_length.0 == remote_file_length {
                        CheckStatus::WorkNotRequired
                    } else {
                        CheckStatus::WorkRequired
                    }
                } else {
                    // Not sure of file length, so we download it.
                    CheckStatus::WorkRequired
                }
            } else {
                // Failed to check. We don't report an error, but maybe we should.
                CheckStatus::WorkRequired
            };

            Ok(check_status)
        })
    }

    fn work_fn<'f>(
        station: &'f mut StationMutRef<'_, DemoError>,
        files: &'f FilesRw,
    ) -> LocalBoxFuture<'f, Result<ResIds, (ResIds, DemoError)>> {
        station.progress.progress_bar().reset();
        station.progress.tick();
        Box::pin(async move {
            let mut res_ids = ResIds::new();
            let client = reqwest::Client::builder()
                .redirect(Policy::none())
                .build()
                .map_err(|error| Self::client_build_error(error))
                .map_err(|e| (res_ids.clone(), e))?;

            let mut files = files.write().await;

            let app_zip_build_agent_path = station.dir.join(APP_ZIP_NAME);
            let app_zip_byte_stream =
                Self::app_zip_read(station, &mut files, &app_zip_build_agent_path)
                    .await
                    .map_err(|e| (res_ids.clone(), e))?;

            let address = Cow::Owned(SERVER_PARAMS_DEFAULT.address());
            let address_file_id = files.add("artifact_server_address", address);
            let address = files.source(address_file_id).clone();

            let form = Form::new().part(
                "files",
                Part::stream(reqwest::Body::wrap_stream(app_zip_byte_stream))
                    .file_name(APP_ZIP_NAME),
            );
            let response = client
                .post(&*address)
                .multipart(form)
                .send()
                .await
                .map_err(|error| {
                    let app_zip_dir_file_id = files.add(
                        station.dir,
                        Cow::Owned(station.dir.to_string_lossy().into_owned()),
                    );
                    StationAErrors::connect_error(
                        &SERVER_PARAMS_DEFAULT,
                        app_zip_dir_file_id,
                        &address,
                        address_file_id,
                        error,
                    )
                })
                .map_err(|e| (res_ids.clone(), e))?;

            let status_code = response.status();
            if status_code.as_u16() == 302 {
                // If the server supported delete, this would be the url.
                let artifact_server_app_zip_url = address + "/" + APP_ZIP_NAME;
                let _ = res_ids.insert(
                    ResIdLogical::new(crate::res_ids::ARTIFACT_SERVER_APP_ZIP),
                    artifact_server_app_zip_url,
                );

                Ok(res_ids)
            } else {
                let address_span = Span::from_str(&address);
                let app_zip_path_file_id = files.add(
                    APP_ZIP_NAME,
                    Cow::Owned(app_zip_build_agent_path.to_string_lossy().into_owned()),
                );
                let app_zip_path = files.source(app_zip_path_file_id);
                let app_zip_path_span = Span::from_str(app_zip_path);
                let server_message = if let Ok(server_message) = response.text().await {
                    Some(server_message)
                } else {
                    // Failed to receive response text.
                    // Ignore why the sub-operation failed, but still report the upload reject.
                    None
                };

                let code = ErrorCode::AppZipReject;
                let detail = ErrorDetail::AppZipReject {
                    app_zip_path_file_id,
                    app_zip_path_span,
                    address_file_id,
                    address_span,
                    server_message,
                };
                Err((res_ids, DemoError::new(code, detail, Severity::Error)))
            }
        })
    }

    async fn app_zip_read(
        station: &mut StationMutRef<'_, DemoError>,
        files: &mut Files,
        app_zip_build_agent_path: &Path,
    ) -> Result<FramedRead<File, BytesCodec>, DemoError> {
        let app_zip_read = File::open(app_zip_build_agent_path)
            .await
            .map_err(|error| {
                let app_zip_dir_file_id = files.add(
                    station.dir,
                    Cow::Owned(station.dir.to_string_lossy().into_owned()),
                );
                let app_zip_path_file_id = files.add(
                    APP_ZIP_NAME,
                    Cow::Owned(app_zip_build_agent_path.to_string_lossy().into_owned()),
                );
                let app_zip_path = files.source(app_zip_path_file_id);
                let app_zip_path_span = Span::from_str(app_zip_path);

                let code = ErrorCode::AppZipOpen;
                let detail = ErrorDetail::AppZipOpen {
                    app_zip_dir_file_id,
                    app_zip_path_file_id,
                    app_zip_path_span,
                    error,
                };

                DemoError::new(code, detail, Severity::Error)
            })?;
        Ok(FramedRead::new(app_zip_read, BytesCodec::new()))
    }

    fn client_build_error(error: reqwest::Error) -> DemoError {
        let code = ErrorCode::ReqwestClientBuild;
        let detail = ErrorDetail::ReqwestClientBuild(error);

        DemoError::new(code, detail, Severity::Bug)
    }

    fn file_open_error(
        station: &mut StationMut<'_, DemoError>,
        files: &mut Files,
        app_zip_build_agent_path: &Path,
        error: std::io::Error,
    ) -> DemoError {
        let app_zip_dir_file_id = files.add(
            station.dir,
            Cow::Owned(station.dir.to_string_lossy().into_owned()),
        );
        let app_zip_path_file_id = files.add(
            APP_ZIP_NAME,
            Cow::Owned(app_zip_build_agent_path.to_string_lossy().into_owned()),
        );
        let app_zip_path = files.source(app_zip_path_file_id);
        let app_zip_path_span = Span::from_str(app_zip_path);

        let code = ErrorCode::AppZipOpen;
        let detail = ErrorDetail::AppZipOpen {
            app_zip_dir_file_id,
            app_zip_path_file_id,
            app_zip_path_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }

    fn file_metadata_error(
        files: &mut Files,
        app_zip_build_agent_path: &Path,
        error: std::io::Error,
    ) -> DemoError {
        let app_zip_path_file_id = files.add(
            APP_ZIP_NAME,
            Cow::Owned(app_zip_build_agent_path.to_string_lossy().into_owned()),
        );
        let app_zip_path = files.source(app_zip_path_file_id);
        let app_zip_path_span = Span::from_str(app_zip_path);

        let code = ErrorCode::WebServerAppZipMetadata;
        let detail = ErrorDetail::WebServerAppZipMetadata {
            app_zip_path_file_id,
            app_zip_path_span,
            error,
        };

        DemoError::new(code, detail, Severity::Error)
    }
}

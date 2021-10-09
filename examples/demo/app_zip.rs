pub const APP_ZIP_NAME: &'static str = "app.zip";
pub const APP_ZIP_BUILD_AGENT_PARENT_PATH: &'static str = "/tmp/choochoo/demo/station_a";
pub const APP_ZIP_BUILD_AGENT_PATH: &'static str = "/tmp/choochoo/demo/station_a/app.zip";
pub const APP_ZIP_ARTIFACT_SERVER_PATH: &'static str = "/tmp/choochoo/demo";
pub const APP_ZIP_APP_SERVER_PARENT: &'static str = "/tmp/choochoo/demo/station_c";
pub const APP_ZIP_APP_SERVER_PATH: &'static str = "/tmp/choochoo/demo/station_c/app.zip";

/// Resource indicating the application's file length.
#[derive(Debug)]
pub struct AppZipFileLength(pub u64);

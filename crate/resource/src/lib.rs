//! Runtime data types for the choochoo automation library.

pub use crate::{
    files::Files, files_rw::FilesRw, profile::Profile, profile_dir::ProfileDir,
    profile_error::ProfileError, station_dir::StationDir, workspace_dir::WorkspaceDir,
};

mod files;
mod files_rw;
mod profile;
mod profile_dir;
mod profile_error;
mod station_dir;
mod workspace_dir;

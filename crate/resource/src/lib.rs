//! Runtime data types for the choochoo automation library.

pub use crate::{
    files::Files, files_rw::FilesRw, profile::Profile, profile_dir::ProfileDir,
    profile_error::ProfileError, workspace_dir::WorkspaceDir,
};

mod files;
mod files_rw;
mod profile;
mod profile_dir;
mod profile_error;
mod workspace_dir;

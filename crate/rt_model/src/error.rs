//! Types representing errors and their details.

use std::{fmt, path::PathBuf};

use tokio::task::JoinError;

use choochoo_cfg_model::{
    rt::{StationDir, TrainReport},
    StationSpec,
};
use choochoo_resource::{ProfileDir, WorkspaceDir};

pub use self::{as_diagnostic::AsDiagnostic, station_spec_error::StationSpecError};

mod as_diagnostic;
mod station_spec_error;

/// Error while using `choochoo`.
#[derive(Debug)]
pub enum Error<E> {
    /// Failed to join the multi-progress bar task.
    MultiProgressTaskJoin(JoinError),
    /// Failed to join the multi-progress bar.
    MultiProgressJoin(std::io::Error),
    /// Failed to create profile directory.
    ProfileDirCreate {
        /// The directory that was attempted to be created.
        profile_dir: ProfileDir,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Failed to create station directory.
    StationDirCreate {
        /// The directory that was attempted to be created.
        station_dir: StationDir,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Station setup failed.
    ///
    /// Details of failures are recorded in the TrainReport instead of this
    /// variant.
    StationSetup {
        /// The train report.
        train_report: TrainReport<E>,
    },
    /// Failed to queue a station for visiting.
    StationQueue {
        /// The specification of the station that failed to be queued.
        station_spec: StationSpec<E>,
    },
    /// Station visitor failed to notify the queuer that a station is completed.
    StationVisitNotify {
        /// The specification of the station whose notification failed to be
        /// sent.
        station_spec: StationSpec<E>,
    },
    /// Failed to read current directory to discover workspace directory.
    WorkingDirRead(std::io::Error),
    /// Failed to create workspace directory.
    WorkspaceDirCreate {
        /// The directory that was attempted to be created.
        workspace_dir: WorkspaceDir,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Failed to find workspace marker file to determine workspace directory.
    WorkspaceFileNotFound {
        /// Beginning directory of traversal.
        working_dir: PathBuf,
        /// File or directory name searched for.
        file_name: PathBuf,
    },
}

impl<E> fmt::Display for Error<E>
where
    E: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MultiProgressTaskJoin(_) => {
                write!(f, "Failed to join the multi-progress bar task.")
            }
            Self::MultiProgressJoin(_) => write!(f, "Failed to join the multi-progress bar."),
            Self::ProfileDirCreate { profile_dir, .. } => write!(
                f,
                "Failed to create profile directory: `{}`.",
                profile_dir.display()
            ),
            Self::StationDirCreate { station_dir, .. } => write!(
                f,
                "Failed to create station directory: `{}`.",
                station_dir.display()
            ),
            Self::StationSetup { .. } => write!(f, "Station setup failed"),
            Self::StationQueue { station_spec } => write!(
                f,
                "Failed to queue station: `{id}: {name}`",
                id = station_spec.id(),
                name = station_spec.name()
            ),
            Self::StationVisitNotify { station_spec } => write!(
                f,
                "Failed to notify completion of station: `{id}: {name}`",
                id = station_spec.id(),
                name = station_spec.name()
            ),
            Self::WorkingDirRead(_) => write!(
                f,
                "Failed to read current directory to discover workspace directory."
            ),
            Self::WorkspaceDirCreate { workspace_dir, .. } => write!(
                f,
                "Failed to create workspace directory: `{}`.",
                workspace_dir.display()
            ),
            Self::WorkspaceFileNotFound {
                working_dir,
                file_name,
            } => write!(
                f,
                "Failed to determine workspace directory as could not find `{file_name}` in `{working_dir}` or any parent directories.",
                file_name = file_name.display(),
                working_dir = working_dir.display(),
            ),
        }
    }
}

impl<E> std::error::Error for Error<E>
where
    E: fmt::Debug + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MultiProgressTaskJoin(error) => Some(error),
            Self::MultiProgressJoin(error) => Some(error),
            Self::ProfileDirCreate { error, .. } => Some(error),
            Self::StationDirCreate { error, .. } => Some(error),
            Self::StationSetup { .. } => None,
            Self::StationQueue { .. } => None,
            Self::StationVisitNotify { .. } => None,
            Self::WorkingDirRead(error) => Some(error),
            Self::WorkspaceDirCreate { error, .. } => Some(error),
            Self::WorkspaceFileNotFound { .. } => None,
        }
    }
}

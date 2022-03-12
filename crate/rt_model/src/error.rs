//! Types representing errors and their details.

use std::{fmt, path::PathBuf};

use tokio::task::JoinError;

use choochoo_cfg_model::{
    rt::{ResIds, StationDir, StationRtId, TrainResources},
    StationId,
};
use choochoo_resource::{HistoryDir, ProfileDir, ProfileHistoryDir, WorkspaceDir};

use crate::ProfileHistoryStationDir;

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
    /// Failed to create history directory.
    HistoryDirCreate {
        /// The directory that was attempted to be created.
        history_dir: HistoryDir,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Failed to create profile directory.
    ProfileDirCreate {
        /// The directory that was attempted to be created.
        profile_dir: ProfileDir,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Failed to create profile history directory.
    ProfileHistoryDirCreate {
        /// The directory that was attempted to be created.
        profile_history_dir: ProfileHistoryDir,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Failed to create profile history station directory.
    ProfileHistoryStationDirCreate {
        /// The directory that was attempted to be created.
        profile_history_station_dir: ProfileHistoryStationDir,
        /// Underlying IO error.
        error: std::io::Error,
    },
    /// Channel receiver for [`ResIds`] produced by stations was closed.
    ///
    /// Should be impossible to hit.
    ResIdsChannelClosed {
        /// Runtime ID of the station when the error occurred.
        station_id: StationId,
        /// Underlying channel send error.
        error: tokio::sync::mpsc::error::SendError<(StationRtId, ResIds)>,
    },
    /// Failed to serialize a resource ID produced by a station.
    ResIdSerialize {
        /// Runtime ID of the station.
        station_id: StationId,
        /// Underlying serialization error.
        error: serde_json::error::Error,
    },
    /// Failed to write [`ResIds`] produced by a station.
    ResIdWrite {
        /// Runtime ID of the station.
        station_id: StationId,
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
    /// Details of failures are recorded in the TrainResources instead of this
    /// variant.
    StationSetup {
        /// The train resources.
        train_resources: TrainResources<E>,
    },
    /// Failed to create target directory.
    TargetDirCreate {
        /// The directory that was attempted to be created.
        target_dir: PathBuf,
        /// Underlying IO error.
        error: std::io::Error,
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
            Self::HistoryDirCreate { history_dir, .. } => write!(
                f,
                "Failed to create history directory: `{}`.",
                history_dir.display()
            ),
            Self::ProfileDirCreate { profile_dir, .. } => write!(
                f,
                "Failed to create profile directory: `{}`.",
                profile_dir.display()
            ),
            Self::ProfileHistoryDirCreate {
                profile_history_dir,
                ..
            } => write!(
                f,
                "Failed to create profile history directory: `{}`.",
                profile_history_dir.display()
            ),
            Self::ProfileHistoryStationDirCreate {
                profile_history_station_dir,
                ..
            } => write!(
                f,
                "Failed to create profile history station directory: `{}`.",
                profile_history_station_dir.display()
            ),
            Self::ResIdsChannelClosed { station_id, .. } => write!(
                f,
                "Channel receiver for `ResIds` produced by stations was closed while sending resource IDs for {station_id}"
            ),
            Self::ResIdSerialize { station_id, .. } => write!(
                f,
                "Failed to serialize resource ID produced by station {station_id}."
            ),
            Self::ResIdWrite { station_id, .. } => write!(
                f,
                "Failed to write `ResIds` produced by station {station_id}."
            ),
            Self::StationDirCreate { station_dir, .. } => write!(
                f,
                "Failed to create station directory: `{}`.",
                station_dir.display()
            ),
            Self::StationSetup { .. } => write!(f, "Station setup failed"),
            Self::TargetDirCreate { target_dir, .. } => write!(
                f,
                "Failed to create target directory: `{}`.",
                target_dir.display()
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
            Self::HistoryDirCreate { error, .. } => Some(error),
            Self::ProfileDirCreate { error, .. } => Some(error),
            Self::ProfileHistoryDirCreate { error, .. } => Some(error),
            Self::ProfileHistoryStationDirCreate { error, .. } => Some(error),
            Self::ResIdsChannelClosed { error, .. } => Some(error),
            Self::ResIdSerialize { error, .. } => Some(error),
            Self::ResIdWrite { error, .. } => Some(error),
            Self::StationDirCreate { error, .. } => Some(error),
            Self::StationSetup { .. } => None,
            Self::TargetDirCreate { error, .. } => Some(error),
            Self::WorkingDirRead(error) => Some(error),
            Self::WorkspaceDirCreate { error, .. } => Some(error),
            Self::WorkspaceFileNotFound { .. } => None,
        }
    }
}

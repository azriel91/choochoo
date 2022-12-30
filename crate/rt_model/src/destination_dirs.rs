use choochoo_resource::{HistoryDir, ProfileDir, ProfileHistoryDir, WorkspaceDir};

use crate::StationDirs;

/// Directories used during `choochoo` execution.
///
/// This is not part of the [`choochoo_resource`] crate as it is not a resource,
/// but rather a grouping of directories produced by [`DestinationDirCalc`].
///
/// [`DestinationDirCalc`]: crate::DestinationDirCalc
#[derive(Clone, Debug)]
pub struct DestinationDirs {
    /// Base directory of the workspace.
    pub workspace_dir: WorkspaceDir,
    /// Directory to store resource IDs produced by different profile
    /// executions.
    pub history_dir: HistoryDir,
    /// Directory to store all resource IDs produced by a profile's executions.
    pub profile_history_dir: ProfileHistoryDir,
    /// Directory to store all data produced by the current profile's execution.
    pub profile_dir: ProfileDir,
    /// Map from [`StationRtId`] to each station's execution directory.
    pub station_dirs: StationDirs,
}

impl DestinationDirs {
    /// Returns a reference to the workspace dir.
    pub fn workspace_dir(&self) -> &WorkspaceDir {
        &self.workspace_dir
    }

    /// Returns a reference to the history dir.
    pub fn history_dir(&self) -> &HistoryDir {
        &self.history_dir
    }

    /// Returns a reference to the profile history dir.
    pub fn profile_history_dir(&self) -> &ProfileHistoryDir {
        &self.profile_history_dir
    }

    /// Returns a reference to the profile dir.
    pub fn profile_dir(&self) -> &ProfileDir {
        &self.profile_dir
    }

    /// Returns a reference to the station dirs.
    pub fn station_dirs(&self) -> &StationDirs {
        &self.station_dirs
    }
}

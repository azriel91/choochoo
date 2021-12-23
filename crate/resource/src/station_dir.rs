use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

/// Directory that records state for each station.
///
/// Information stored in this directory should be able to be reused in
/// subsequent executions or simply for reporting -- e.g. last execution status.
pub struct StationDir(PathBuf);

impl Deref for StationDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

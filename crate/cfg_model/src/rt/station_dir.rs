use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

/// Directory to hold data specific to each station.
///
/// Information stored in this directory should be able to be reused in
/// subsequent executions or simply for reporting -- e.g. last execution status.
#[derive(Clone, Debug, PartialEq)]
pub struct StationDir(PathBuf);

impl StationDir {
    /// Returns a new [`StationDir`].
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl Deref for StationDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

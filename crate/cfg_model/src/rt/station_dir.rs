use std::{
    ffi::OsStr,
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

impl AsRef<OsStr> for StationDir {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for StationDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for StationDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

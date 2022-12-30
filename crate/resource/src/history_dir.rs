use std::{
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
};

/// Directory to store resource IDs produced by different profile executions.
///
/// Typically `${workspace}/target/.history`.
///
/// The profile history directory is intended to be version controllable, and
/// managed independently of execution artifacts. It is also separate from the
/// profile directory so that removing the profile directory for a clean
/// deployment does not accidentally remove information needed to clean up
/// resources from previous executions.
#[derive(Clone, Debug, PartialEq)]
pub struct HistoryDir(PathBuf);

impl HistoryDir {
    /// Returns a new [`HistoryDir`].
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<OsStr> for HistoryDir {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for HistoryDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for HistoryDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

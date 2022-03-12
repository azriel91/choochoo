use std::{
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
};

/// Directory to store all resource IDs produced by a profile's executions.
///
/// Typically `${workspace}/target/.history/${profile}`.
///
/// The profile history directory is intended to be version controllable, and
/// managed independently of execution artifacts. It is also separate from the
/// profile directory so that removing the profile directory for a clean
/// deployment does not accidentally remove information needed to clean up
/// resources from previous executions.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileHistoryDir(PathBuf);

impl ProfileHistoryDir {
    /// Returns a new [`ProfileHistoryDir`].
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<OsStr> for ProfileHistoryDir {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for ProfileHistoryDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for ProfileHistoryDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use std::{
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
};

/// Directory that stores the resource IDs for a station.
///
/// This type itself is not a resource, but is within the
/// [`ProfileHistoryStationDirs`] resource.
///
/// [`ProfileHistoryStationDirs`]: crate::ProfileHistoryStationDirs
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileHistoryStationDir(PathBuf);

impl ProfileHistoryStationDir {
    /// Returns a new [`ProfileHistoryStationDir`].
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<OsStr> for ProfileHistoryStationDir {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for ProfileHistoryStationDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for ProfileHistoryStationDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

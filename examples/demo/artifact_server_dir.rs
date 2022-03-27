use std::{
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
};

/// Directory where `app.zip` is uploaded to.
#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactServerDir(PathBuf);

impl ArtifactServerDir {
    /// Returns a new [`ArtifactServerDir`].
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<OsStr> for ArtifactServerDir {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for ArtifactServerDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for ArtifactServerDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

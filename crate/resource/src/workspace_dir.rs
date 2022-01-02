use std::{
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
};

/// Base directory of the workspace.
///
/// Given a workspace lives in `workspace_dir`, it is natural for users to
/// execute `choochoo` in any sub directory of `workspace_dir`, in which case
/// execution should be consistent with invocations in `workspace_dir`.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceDir(PathBuf);

impl WorkspaceDir {
    /// Returns a new [`WorkspaceDir`].
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<OsStr> for WorkspaceDir {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for WorkspaceDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for WorkspaceDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

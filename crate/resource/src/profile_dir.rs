use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

/// Directory to store all data produced by an execution.
///
/// This is the directory that physically and logically contains all information
/// produced and used during a `choochoo` invocation. Exceptions include
/// authentication information stored in their respective directories on the
/// file system, such as application credentials stored in
/// `~/${app}/credentials`.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileDir(PathBuf);

impl ProfileDir {
    /// Returns a new [`ProfileDir`].
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for ProfileDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for ProfileDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

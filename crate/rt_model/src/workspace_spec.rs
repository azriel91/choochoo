use std::path::Path;

/// Describes how to discover the workspace directory.
#[derive(Clone, Debug, PartialEq)]
pub enum WorkspaceSpec {
    /// Use the exe working directory as the workspace directory.
    ///
    /// The working directory is the directory that the user ran the program in.
    WorkingDir,
    /// Traverse up from the working directory until the given file is found.
    ///
    /// The workspace directory is the parent directory that contains a file or
    /// directory with the provided name.
    FirstDirWithFile(&'static Path),
}

impl Default for WorkspaceSpec {
    fn default() -> Self {
        Self::WorkingDir
    }
}

/// How task execution should be structured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyMode {
    /// Run tasks sequentially.
    Sequential,
    /// Run tasks using their logical dependencies.
    Concurrent,
}

/// Whether the station was changed when visiting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnsureOutcome {
    /// The station was already in the desired state.
    Unchanged,
    /// The station was visited.
    Changed,
}

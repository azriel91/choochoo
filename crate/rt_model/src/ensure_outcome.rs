/// Ensure outcome is `Ok`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnsureOutcomeOk {
    /// The station was already in the desired state.
    Unchanged,
    /// The station was visited.
    Changed,
}

/// Ensure outcome is an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnsureOutcomeErr<E> {
    /// The station's check function failed.
    CheckFail(E),
    /// The station's visit function failed.
    VisitFail(E),
}

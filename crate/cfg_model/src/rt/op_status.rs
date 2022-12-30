/// Status of an operation's execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OpStatus {
    /// Operation setup function has not been run.
    SetupQueued,
    /// Operation setup function ran successfully.
    SetupSuccess,
    /// Operation setup function failed.
    SetupFail,
    /// Operation has at least one parent that hasn't been executed.
    ParentPending,
    /// At least one of this operation's parents failed to be executed.
    ///
    /// There will not be an attempt to visit this operation.
    ParentFail,
    /// Operation is ready to be executed, but has not been.
    OpQueued,
    /// Operation check function failed.
    CheckFail,
    /// Work execution is in progress.
    WorkInProgress,
    /// The work was not necessary to be executed.
    WorkUnnecessary,
    /// The work has been successfully executed.
    WorkSuccess,
    /// The work execution failed.
    WorkFail,
}

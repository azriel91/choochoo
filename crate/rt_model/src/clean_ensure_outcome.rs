use choochoo_cfg_model::resman::BorrowFail;

use crate::error::StationSpecError;

/// Ensure outcome is `Ok`.
#[derive(Clone, Debug)]
pub enum CleanEnsureOutcomeOk {
    /// The station does not need to be cleaned.
    ///
    /// This usually means the station does not create any resources, or a
    /// predecessor station's clean encompasses cleaning up the changes of this
    /// station.
    ///
    /// For example, a station that modifies configuration of a web application
    /// will be cleaned by a station whose clean operation is to remove the web
    /// application installation.
    NothingToDo,
    /// The station was already cleaned.
    Unchanged,
    /// The station was visited.
    Changed {
        /// Whether any error with the operation is detected.
        ///
        /// If the operation is successfully executed, but the check function
        /// reports it still needs work, then there is potentially a bug in the
        /// station spec.
        station_spec_error: Option<StationSpecError>,
    },
}

/// Ensure outcome is an error.
#[derive(Clone, Debug)]
pub enum CleanEnsureOutcomeErr<E> {
    /// Impossible to hit.
    ///
    /// This models when the clean work function is invoked, but the clean
    /// operation doesn't actually exist.
    Never,
    /// Failed to borrow resources for the check function.
    ///
    /// Usually this implies the resource was not inserted in the setup
    /// function.
    CheckBorrowFail(BorrowFail),
    /// The operation's check function failed.
    CheckFail(E),
    /// Failed to borrow resources for the check function.
    ///
    /// Usually this implies the resource was not inserted in the setup
    /// function, or a previous station did not correctly insert a resource.
    VisitBorrowFail(BorrowFail),
    /// The operation's work function failed.
    WorkFail {
        /// The visit error.
        error: E,
    },
}

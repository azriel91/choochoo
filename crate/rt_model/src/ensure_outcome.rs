use choochoo_cfg_model::resman::BorrowFail;

use crate::error::StationSpecError;

/// Ensure outcome is `Ok`.
#[derive(Clone, Debug, PartialEq)]
pub enum EnsureOutcomeOk {
    /// The station was already in the desired state.
    Unchanged,
    /// The station was visited.
    Changed {
        /// Whether any error with the station spec is detected.
        ///
        /// If the station is successfully visited, but the check function
        /// reports it still needs to be visited, then there is potentially a
        /// bug in the station spec.
        station_spec_error: Option<StationSpecError>,
    },
}

/// Ensure outcome is an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnsureOutcomeErr<E> {
    /// Failed to borrow resources for the check function.
    ///
    /// Usually this implies the resource was not inserted in the setup
    /// function.
    CheckBorrowFail(BorrowFail),
    /// The station's check function failed.
    CheckFail(E),
    /// Failed to borrow resources for the check function.
    ///
    /// Usually this implies the resource was not inserted in the setup
    /// function, or a previous station did not correctly insert a resource.
    VisitBorrowFail(BorrowFail),
    /// The station's visit function failed.
    VisitFail(E),
}

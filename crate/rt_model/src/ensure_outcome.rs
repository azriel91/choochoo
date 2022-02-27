use choochoo_cfg_model::{resman::BorrowFail, rt::ResourceIds};

use crate::error::StationSpecError;

/// Ensure outcome is `Ok`.
#[derive(Clone, Debug)]
pub enum EnsureOutcomeOk {
    /// The station was already in the desired state.
    Unchanged,
    /// The station was visited.
    Changed {
        /// Resource IDs generated during the visit.
        resource_ids: ResourceIds,
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
pub enum EnsureOutcomeErr<E> {
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
        /// Resource IDs generated during the visit.
        resource_ids: ResourceIds,
        /// The visit error.
        error: E,
    },
}

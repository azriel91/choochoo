use fn_graph::{FnMeta, TypeIds};

use crate::{rt::CheckStatus, SetupFn, StationFn};

// **Note:** `Clone` is manually implemented to avoid the trait bound on `E`.
/// Grouping of a station's behaviours.
#[derive(Debug, PartialEq)]
pub struct OpFns<E> {
    /// Verifies input, calculates progress limit, and inserts resources.
    pub setup_fn: SetupFn<E>,
    /// Checks whether the operation needs to be executed.
    ///
    /// If this is `None`, then the operation will always be executed.
    ///
    /// This is run before and after `work_fn` is executed.
    pub check_fn: Option<StationFn<CheckStatus, E>>,
    /// Steps to execute when visiting a station.
    pub work_fn: StationFn<(), E>,
}

impl<E> OpFns<E> {
    /// Returns new `OpFns` with minimal logic.
    pub fn new(setup_fn: SetupFn<E>, work_fn: StationFn<(), E>) -> Self {
        Self {
            setup_fn,
            check_fn: None,
            work_fn,
        }
    }

    /// Sets the `check_fn` for this `OpFns`.
    #[must_use]
    pub fn with_check_fn(mut self, check_fn: StationFn<CheckStatus, E>) -> Self {
        self.check_fn = Some(check_fn);
        self
    }
}

impl<E> Clone for OpFns<E> {
    fn clone(&self) -> Self {
        Self {
            setup_fn: self.setup_fn.clone(),
            check_fn: self.check_fn.clone(),
            work_fn: self.work_fn.clone(),
        }
    }
}

impl<E> FnMeta for OpFns<E> {
    fn borrows(&self) -> TypeIds {
        self.work_fn.borrows()
    }

    fn borrow_muts(&self) -> TypeIds {
        self.work_fn.borrow_muts()
    }
}

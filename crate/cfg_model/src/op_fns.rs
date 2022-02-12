use fn_graph::{FnMeta, TypeIds};

use crate::{rt::CheckStatus, SetupFn, StationFn};

// **Note:** `Clone` and `PartialEq` are manually implemented to avoid the trait
// bound on `Ret` and `E`.
/// Grouping of a station's behaviours.
#[derive(Debug)]
pub struct OpFns<Ret, E> {
    /// Verifies input, calculates progress limit, and inserts resources.
    pub setup_fn: SetupFn<E>,
    /// Checks whether the operation needs to be executed.
    ///
    /// If this is `None`, then the operation will always be executed.
    ///
    /// This is run before and after `work_fn` is executed.
    pub check_fn: Option<StationFn<Result<CheckStatus, E>, E>>,
    /// Steps to execute when visiting a station.
    pub work_fn: StationFn<Ret, E>,
}

impl<Ret, E> OpFns<Ret, E> {
    /// Returns new `OpFns` with minimal logic.
    pub fn new(setup_fn: SetupFn<E>, work_fn: StationFn<Ret, E>) -> Self {
        Self {
            setup_fn,
            check_fn: None,
            work_fn,
        }
    }

    /// Sets the `check_fn` for this `OpFns`.
    #[must_use]
    pub fn with_check_fn(mut self, check_fn: StationFn<Result<CheckStatus, E>, E>) -> Self {
        self.check_fn = Some(check_fn);
        self
    }
}

impl<Ret, E> Clone for OpFns<Ret, E> {
    fn clone(&self) -> Self {
        Self {
            setup_fn: self.setup_fn.clone(),
            check_fn: self.check_fn.clone(),
            work_fn: self.work_fn.clone(),
        }
    }
}

impl<Ret, E> PartialEq for OpFns<Ret, E> {
    fn eq(&self, other: &Self) -> bool {
        self.setup_fn.eq(&other.setup_fn)
            && self.check_fn.eq(&other.check_fn)
            && self.work_fn.eq(&other.work_fn)
    }
}

impl<Ret, E> FnMeta for OpFns<Ret, E> {
    fn borrows(&self) -> TypeIds {
        self.work_fn.borrows()
    }

    fn borrow_muts(&self) -> TypeIds {
        self.work_fn.borrow_muts()
    }
}

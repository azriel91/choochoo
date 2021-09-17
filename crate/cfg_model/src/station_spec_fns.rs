use crate::{CheckStatus, StationFn};

// **Note:** `Clone` is manually implemented to avoid the trait bound on `E`.
/// Grouping of a station's behaviours.
#[derive(Debug, PartialEq)]
pub struct StationSpecFns<E> {
    /// Checks whether a station needs to be visited.
    ///
    /// This is run before and after `visit_fn` is executed.
    pub check_fn: Option<StationFn<CheckStatus, E>>,
    /// Steps to execute when visiting a station.
    pub visit_fn: StationFn<(), E>,
}

impl<E> StationSpecFns<E> {
    /// Returns new `StationSpecFns` with minimal logic.
    pub fn new(visit_fn: StationFn<(), E>) -> Self {
        Self {
            check_fn: None,
            visit_fn,
        }
    }

    /// Sets the `check_fn` for this `StationSpecFns`.
    pub fn with_check_fn(mut self, check_fn: StationFn<CheckStatus, E>) -> Self {
        self.check_fn = Some(check_fn);
        self
    }
}

impl<E> Clone for StationSpecFns<E> {
    fn clone(&self) -> Self {
        Self {
            check_fn: self.check_fn.clone(),
            visit_fn: self.visit_fn.clone(),
        }
    }
}

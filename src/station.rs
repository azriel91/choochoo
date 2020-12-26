use std::{future::Future, pin::Pin};

use crate::{VisitFn, VisitStatus};

/// A state along the way to the destination.
///
/// This is a high level item that is included in the user facing progress
/// report.
#[derive(Clone, Debug, PartialEq)]
pub struct Station {
    /// Whether this station has been visited.
    pub visit_status: VisitStatus,
    /// Steps to run when this station is visited.
    visit_fn: VisitFn,
}

impl Station {
    /// Returns a new [`Station`].
    ///
    /// # Parameters
    ///
    /// * `visit_status`: Whether this [`Station`] is ready to be visited.
    /// * `visit_fn`: Steps to run when this station is visited.
    pub fn new(visit_status: VisitStatus, visit_fn: VisitFn) -> Self {
        Self {
            visit_status,
            visit_fn,
        }
    }

    /// Returns a station visitation pass.
    pub fn visit(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        (self.visit_fn.0)(self)
    }
}

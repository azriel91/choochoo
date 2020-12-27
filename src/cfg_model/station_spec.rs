use crate::VisitFn;

/// Behaviour specification for a station.
#[derive(Clone, Debug, PartialEq)]
pub struct StationSpec {
    /// Steps to run when this station is visited.
    visit_fn: VisitFn,
}

impl StationSpec {
    /// Returns a new [`Station`].
    ///
    /// # Parameters
    ///
    /// * `visit_fn`: Steps to run when this station is visited.
    pub fn new(visit_fn: VisitFn) -> Self {
        Self { visit_fn }
    }

    /// Returns a station visitation pass.
    pub fn visit_fn(&self) -> VisitFn {
        self.visit_fn
    }
}

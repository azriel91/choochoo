use crate::cfg_model::VisitFn;

/// Behaviour specification for a station.
#[derive(Clone, Debug, PartialEq)]
pub struct StationSpec<E> {
    /// Steps to run when this station is visited.
    visit_fn: VisitFn<E>,
}

impl<E> StationSpec<E> {
    /// Returns a new [`Station`].
    ///
    /// # Parameters
    ///
    /// * `visit_fn`: Steps to run when this station is visited.
    pub fn new(visit_fn: VisitFn<E>) -> Self {
        Self { visit_fn }
    }

    /// Returns a station visitation pass.
    pub fn visit_fn(&self) -> VisitFn<E> {
        self.visit_fn
    }
}

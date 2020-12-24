use crate::VisitStatus;

/// A state along the way to the destination.
///
/// This is a high level item that is included in the user facing progress
/// report.
#[derive(Clone, Debug)]
pub struct Station {
    /// Whether this station has been visited.
    visit_status: VisitStatus,
}

impl Station {
    /// Returns a new [`Station`].
    pub fn new(visit_status: VisitStatus) -> Self {
        Self { visit_status }
    }

    /// Returns whether this [`Station`] has been visited.
    pub fn visit_status(&self) -> VisitStatus {
        self.visit_status
    }

    /// Returns a station visitation pass.
    pub fn visit(&mut self) {
        self.visit_status = VisitStatus::Visited
    }
}

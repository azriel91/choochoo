use std::fmt;

use crate::cfg_model::{StationId, VisitFn};

/// Behaviour specification for a station.
#[derive(Clone, Debug, PartialEq)]
pub struct StationSpec<E> {
    /// Unique identifier of the station.
    id: StationId,
    /// Human readable name of the station.
    name: String,
    /// Short description of the station's purpose.
    description: String,
    /// Steps to run when this station is visited.
    visit_fn: VisitFn<E>,
}

impl<E> StationSpec<E> {
    /// Returns a new [`Station`].
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    /// * `name`: Human readable name of the station.
    /// * `description`: Short description of the station's purpose.
    /// * `visit_fn`: Steps to run when this station is visited.
    pub fn new(id: StationId, name: String, description: String, visit_fn: VisitFn<E>) -> Self {
        Self {
            id,
            name,
            description,
            visit_fn,
        }
    }

    /// Returns a station visitation pass.
    pub fn visit_fn(&self) -> VisitFn<E> {
        self.visit_fn.clone()
    }
}

impl<E> fmt::Display for StationSpec<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

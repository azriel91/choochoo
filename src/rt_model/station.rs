use std::fmt;

use crate::{
    cfg_model::{StationSpec, VisitFnReturn},
    rt_model::VisitStatus,
};

/// A state along the way to the destination.
///
/// This is a high level item that is included in the user facing progress
/// report.
#[derive(Clone, Debug, PartialEq)]
pub struct Station<E> {
    /// Behaviour specification for this station.
    pub station_spec: StationSpec<E>,
    /// Whether this station has been visited.
    pub visit_status: VisitStatus,
}

impl<E> Station<E> {
    /// Returns a new [`Station`].
    ///
    /// # Parameters
    ///
    /// * `station_spec`: Behaviour specification for this station.
    /// * `visit_status`: Whether this [`Station`] is ready to be visited.
    pub fn new(station_spec: StationSpec<E>, visit_status: VisitStatus) -> Self {
        Self {
            station_spec,
            visit_status,
        }
    }

    /// Returns a station visitation pass.
    pub fn visit(&mut self) -> VisitFnReturn<'_, E> {
        (self.station_spec.visit_fn().0)(self)
    }
}

impl<E> fmt::Display for Station<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] ", self.visit_status)?;

        self.station_spec.fmt(f)
    }
}

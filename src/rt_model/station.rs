use std::{future::Future, pin::Pin};

use crate::{cfg_model::StationSpec, VisitStatus};

/// A state along the way to the destination.
///
/// This is a high level item that is included in the user facing progress
/// report.
#[derive(Clone, Debug, PartialEq)]
pub struct Station {
    /// Behaviour specification for this station.
    pub station_spec: StationSpec,
    /// Whether this station has been visited.
    pub visit_status: VisitStatus,
}

impl Station {
    /// Returns a new [`Station`].
    ///
    /// # Parameters
    ///
    /// * `station_spec`: Behaviour specification for this station.
    /// * `visit_status`: Whether this [`Station`] is ready to be visited.
    pub fn new(station_spec: StationSpec, visit_status: VisitStatus) -> Self {
        Self {
            station_spec,
            visit_status,
        }
    }

    /// Returns a station visitation pass.
    pub fn visit(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        (self.station_spec.visit_fn().0)(self)
    }
}

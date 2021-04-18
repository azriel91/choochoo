use std::fmt;

use futures::Future;
use resman::Resources;

use crate::{cfg_model::StationSpec, rt_model::VisitStatus};

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
    pub fn visit<'f>(
        &'f mut self,
        resources: &'f Resources,
    ) -> impl Future<Output = Result<(), E>> + 'f {
        let visit_fn = self.station_spec.station_spec_fns().visit_fn.clone();
        visit_fn.0(self, resources)
    }
}

impl<E> fmt::Display for Station<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] ", self.visit_status)?;

        self.station_spec.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::Station;
    use crate::{
        cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns},
        rt_model::VisitStatus,
    };

    #[test]
    fn display_returns_readable_informative_message() -> Result<(), StationIdInvalidFmt<'static>> {
        let station_id = StationId::new("station_id")?;
        let name = String::from("Station Name");
        let description = String::from("One liner.");
        let station_spec_fns = {
            let visit_fn = StationFn::new(|_, _| Box::pin(async { Result::<(), ()>::Ok(()) }));
            StationSpecFns::new(visit_fn)
        };
        let station_spec = StationSpec::new(station_id, name, description, station_spec_fns);
        let station = Station::new(station_spec, VisitStatus::InProgress);

        assert_eq!("[InProgress] Station Name: One liner.", station.to_string());
        Ok(())
    }
}

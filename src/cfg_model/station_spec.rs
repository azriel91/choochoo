use std::fmt;

use crate::cfg_model::{StationId, StationSpecFns};

/// Behaviour specification for a station.
#[derive(Debug, Clone, PartialEq)]
pub struct StationSpec<E> {
    /// Unique identifier of the station.
    id: StationId,
    /// Human readable name of the station.
    name: String,
    /// Short description of the station's purpose.
    description: String,
    /// Steps to run when this station is visited.
    station_spec_fns: StationSpecFns<E>,
}

impl<E> StationSpec<E> {
    /// Returns a new [`Station`].
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    /// * `name`: Human readable name of the station.
    /// * `description`: Short description of the station's purpose.
    /// * `station_spec_fns`: Steps to run when this station is visited.
    pub fn new(
        id: StationId,
        name: String,
        description: String,
        station_spec_fns: StationSpecFns<E>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            station_spec_fns,
        }
    }

    /// Returns the unique identifier of the station.
    pub fn id(&self) -> &StationId {
        &self.id
    }

    /// Returns the human readable name of the station.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the short description of the station's purpose.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns this station's behaviours.
    pub fn station_spec_fns(&self) -> &StationSpecFns<E> {
        &self.station_spec_fns
    }
}

impl<E> fmt::Display for StationSpec<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::StationSpec;
    use crate::cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpecFns};

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

        assert_eq!("Station Name: One liner.", station_spec.to_string());
        Ok(())
    }
}

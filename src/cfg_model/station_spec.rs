use std::fmt;

use crate::cfg_model::{StationFn, StationId};

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
    visit_fn: StationFn<E>,
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
    pub fn new(id: StationId, name: String, description: String, visit_fn: StationFn<E>) -> Self {
        Self {
            id,
            name,
            description,
            visit_fn,
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

    /// Returns the steps to run when this station is visited.
    pub fn visit_fn(&self) -> StationFn<E> {
        self.visit_fn.clone()
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
    use crate::cfg_model::{StationFn, StationId, StationIdInvalidFmt};

    #[test]
    fn display_returns_readable_informative_message() -> Result<(), StationIdInvalidFmt<'static>> {
        let station_id = StationId::new("station_id")?;
        let name = String::from("Station Name");
        let description = String::from("One liner.");
        let visit_fn = StationFn::new(|_station| Box::pin(async move { Result::<(), ()>::Ok(()) }));
        let station_spec = StationSpec::new(station_id, name, description, visit_fn);

        assert_eq!("Station Name: One liner.", station_spec.to_string());
        Ok(())
    }
}

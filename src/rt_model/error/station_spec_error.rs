use std::fmt;

use crate::cfg_model::StationId;

/// There is a bug with the station specification.
#[derive(Clone, Debug, PartialEq)]
pub enum StationSpecError {
    /// The `check_fn` provided in the station spec functions returned
    /// [`CheckStatus::VisitRequired`] after the station was visited.
    ///
    /// [`CheckStatus::VisitRequired`]:
    /// crate::cfg_model::CheckStatus::VisitRequired
    VisitRequiredAfterVisit {
        /// Unique identifier of the station.
        id: StationId,
        /// Human readable name of the station.
        name: String,
    },
}

impl fmt::Display for StationSpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::VisitRequiredAfterVisit { id, name } => write!(
                f,
                "Station `{id}: {name}`'s check function reported the station still requires a visit after the visit function was run.",
                id = id,
                name = name
            ),
        }
    }
}

impl std::error::Error for StationSpecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::VisitRequiredAfterVisit { .. } => None,
        }
    }
}

impl From<StationSpecError> for () {
    fn from(_: StationSpecError) {}
}

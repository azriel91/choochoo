use std::fmt;

use choochoo_cfg_model::StationId;

/// There is a bug with the station specification.
#[derive(Clone, Debug, PartialEq)]
pub enum StationSpecError {
    /// The `check_fn` provided in the station spec functions returned
    /// [`CheckStatus::WorkRequired`] after the work was executed.
    ///
    /// [`CheckStatus::WorkRequired`]: choochoo_cfg_model::CheckStatus::WorkRequired
    WorkRequiredAfterVisit {
        /// Unique identifier of the station.
        id: StationId,
        /// Human readable name of the station.
        name: String,
    },
}

impl fmt::Display for StationSpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WorkRequiredAfterVisit { id, name } => write!(
                f,
                "Station `{id}: {name}`'s check function reported the station still requires work after the work function was run."
            ),
        }
    }
}

impl std::error::Error for StationSpecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::WorkRequiredAfterVisit { .. } => None,
        }
    }
}

impl From<StationSpecError> for () {
    fn from(_: StationSpecError) {}
}

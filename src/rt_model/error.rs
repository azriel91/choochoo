//! Types representing errors and their details.

use std::fmt;

use tokio::task::JoinError;

use crate::cfg_model::StationSpec;

pub use self::{as_diagnostic::AsDiagnostic, station_spec_error::StationSpecError};

mod as_diagnostic;
mod station_spec_error;

/// Error while using `choochoo`.
#[derive(Debug)]
pub enum Error<E> {
    /// Failed to join the multi-progress bar task.
    MultiProgressJoin(JoinError),
    /// Failed to queue a station for visiting.
    StationQueue {
        /// The specification of the station that failed to be queued.
        station_spec: StationSpec<E>,
    },
    /// Station visitor failed to notify the queuer that a station is completed.
    StationVisitNotify {
        /// The specification of the station whose notification failed to be
        /// sent.
        station_spec: StationSpec<E>,
    },
}

impl<E> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MultiProgressJoin(_) => write!(f, "Failed to join the multi-progress bar task."),
            Self::StationQueue { station_spec } => write!(
                f,
                "Failed to queue station: `{id}: {name}`",
                id = station_spec.id(),
                name = station_spec.name()
            ),
            Self::StationVisitNotify { station_spec } => write!(
                f,
                "Failed to notify completion of station: `{id}: {name}`",
                id = station_spec.id(),
                name = station_spec.name()
            ),
        }
    }
}

impl<E> std::error::Error for Error<E>
where
    E: fmt::Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MultiProgressJoin(error) => Some(error),
            Self::StationQueue { .. } => None,
            Self::StationVisitNotify { .. } => None,
        }
    }
}

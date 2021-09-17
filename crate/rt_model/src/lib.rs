//! Runtime data types for the choochoo automation library.
//!
//! Runtime data when a train plan is executed. Types in this module are
//! analogous to build artifacts.

pub use daggy;
pub use indexmap;
pub use srcerr;

pub use crate::{
    destination::Destination,
    ensure_outcome::{EnsureOutcomeErr, EnsureOutcomeOk},
    error::Error,
    files::{Files, RwFiles},
    station::Station,
    station_errors::StationErrors,
    station_mut::StationMut,
    station_progresses::StationProgresses,
    station_rt_id::StationRtId,
    station_specs::{StationSpecs, StationsFrozen},
    train_report::TrainReport,
};

pub mod error;

mod destination;
mod ensure_outcome;
mod files;
mod station;
mod station_errors;
mod station_mut;
mod station_progresses;
mod station_rt_id;
mod station_specs;
mod train_report;

//! Runtime data types for the choochoo automation library.
//!
//! Runtime data when a train plan is executed. Types in this module are
//! analogous to build artifacts.

pub use indexmap;
pub use srcerr;

pub use crate::{
    destination::Destination,
    destination_builder::DestinationBuilder,
    ensure_outcome::{EnsureOutcomeErr, EnsureOutcomeOk},
    error::Error,
    files::{Files, RwFiles},
    station::Station,
    station_errors::StationErrors,
    station_mut::StationMut,
    station_progresses::StationProgresses,
    station_rt_id::StationRtId,
    train_report::TrainReport,
};

pub mod error;

mod destination;
mod destination_builder;
mod ensure_outcome;
mod files;
mod station;
mod station_errors;
mod station_mut;
mod station_progresses;
mod station_rt_id;
mod train_report;

//! Runtime data types for the choochoo automation library.
//!
//! Runtime data when a train plan is executed. Types in this module are
//! analogous to build artifacts.

pub use crate::{
    destination::Destination,
    destination_builder::DestinationBuilder,
    destination_dir_calc::DestinationDirCalc,
    ensure_outcome::{EnsureOutcomeErr, EnsureOutcomeOk},
    error::Error,
    station_dirs::StationDirs,
    station_progresses::StationProgresses,
    train_report::TrainReport,
    workspace_spec::WorkspaceSpec,
};

pub mod error;

mod destination;
mod destination_builder;
mod destination_dir_calc;
mod ensure_outcome;
mod station_dirs;
mod station_progresses;
mod train_report;
mod workspace_spec;

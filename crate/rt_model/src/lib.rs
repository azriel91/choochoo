//! Runtime data types for the choochoo automation library.
//!
//! Runtime data when a train plan is executed. Types in this module are
//! analogous to build artifacts.

pub use crate::{
    clean_ensure_outcome::{CleanEnsureOutcomeErr, CleanEnsureOutcomeOk},
    create_ensure_outcome::{CreateEnsureOutcomeErr, CreateEnsureOutcomeOk},
    destination::Destination,
    destination_builder::DestinationBuilder,
    destination_dir_calc::DestinationDirCalc,
    destination_dirs::DestinationDirs,
    error::Error,
    station_dirs::StationDirs,
    station_progresses::StationProgresses,
    train_report::TrainReport,
    workspace_spec::WorkspaceSpec,
};

pub mod error;

mod clean_ensure_outcome;
mod create_ensure_outcome;
mod destination;
mod destination_builder;
mod destination_dir_calc;
mod destination_dirs;
mod station_dirs;
mod station_progresses;
mod train_report;
mod workspace_spec;

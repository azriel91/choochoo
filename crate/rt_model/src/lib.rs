//! Runtime data types for the choochoo automation library.
//!
//! Runtime data when a train plan is executed. Types in this module are
//! analogous to build artifacts.

pub use crate::{
    destination::Destination,
    destination_builder::DestinationBuilder,
    ensure_outcome::{EnsureOutcomeErr, EnsureOutcomeOk},
    error::Error,
    station_progresses::StationProgresses,
    workspace_spec::WorkspaceSpec,
};

pub mod error;

mod destination;
mod destination_builder;
mod ensure_outcome;
mod station_progresses;
mod workspace_spec;

//! Runtime data when a train plan is executed.
//!
//! Types in this module are analogous to build artifacts.

pub use self::{
    destination::Destination,
    ensure_outcome::{EnsureOutcomeErr, EnsureOutcomeOk},
    files::{Files, RwFiles},
    station::Station,
    station_progresses::StationProgresses,
    station_rt_id::StationRtId,
    stations::{Stations, StationsFrozen},
    train_report::TrainReport,
    visit_status::VisitStatus,
};

pub mod error;

mod destination;
mod ensure_outcome;
mod files;
mod station;
mod station_progresses;
mod station_rt_id;
mod stations;
mod train_report;
mod visit_status;

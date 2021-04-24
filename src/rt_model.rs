//! Runtime data when a train plan is executed.
//!
//! Types in this module are analogous to build artifacts.

pub use self::{
    destination::{Destination, StationsQueuedIter},
    files::Files,
    station::Station,
    stations::{Stations, StationsFrozen},
    train_report::TrainReport,
    visit_status::VisitStatus,
};

pub mod error;

mod destination;
mod files;
mod station;
mod stations;
mod train_report;
mod visit_status;

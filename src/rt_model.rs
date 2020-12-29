//! Runtime data when a train plan is executed.
//!
//! Types in this module are analogous to build artifacts.

pub use self::{
    destination::Destination, station::Station, stations::Stations, train_report::TrainReport,
    visit_status::VisitStatus,
};

mod destination;
mod station;
mod stations;
mod train_report;
mod visit_status;

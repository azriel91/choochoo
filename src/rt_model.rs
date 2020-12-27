//! Runtime data when a train plan is executed.
//!
//! Types in this module are analogous to build artifacts.

pub use self::{
    destination::Destination, station::Station, stations::Stations, visit_status::VisitStatus,
};

mod destination;
mod station;
mod stations;
mod visit_status;

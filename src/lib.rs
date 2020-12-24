#![deny(missing_docs, missing_debug_implementations)]

//! Automation that starts where it stops.

pub use crate::{
    destination::Destination, station::Station, stations::Stations, train::Train,
    visit_status::VisitStatus, workload::Workload,
};

mod destination;
mod station;
mod stations;
mod train;
mod visit_status;
mod workload;

#![deny(missing_docs, missing_debug_implementations)]

//! Automation that starts where it stops.

pub use crate::{
    destination::Destination, stations::Stations, train::Train, visit_fn::VisitFn,
    visit_status::VisitStatus, workload::Workload,
};

pub mod cfg_model;
pub mod rt_model;

mod destination;
mod stations;
mod train;
mod visit_fn;
mod visit_status;
mod workload;

//! Runtime visit logic for the choochoo automation library.

pub use crate::{driver::Driver, visit_status_updater::VisitStatusUpdater};

pub mod strategy;

mod driver;
mod visit_status_updater;

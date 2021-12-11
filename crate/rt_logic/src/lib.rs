//! Runtime visit logic for the choochoo automation library.

pub use crate::{driver::Driver, train::Train, visit_status_updater::VisitStatusUpdater};

mod driver;
mod train;
mod visit_status_updater;

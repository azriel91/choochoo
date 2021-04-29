//! Logic to update runtime data.

pub use self::{driver::Driver, visit_status_updater::VisitStatusUpdater};

pub mod strategy;

mod driver;
mod visit_status_updater;

#![deny(missing_docs, missing_debug_implementations)]

//! Automation that starts where it stops.

pub use crate::{destination::Destination, train::Train};

mod destination;
mod train;

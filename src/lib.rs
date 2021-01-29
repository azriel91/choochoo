#![deny(missing_docs, missing_debug_implementations)]

//! Automation that starts where it stops.

pub use crate::train::Train;

pub mod cfg_model;
pub mod fmt;
pub mod rt_logic;
pub mod rt_model;

mod train;

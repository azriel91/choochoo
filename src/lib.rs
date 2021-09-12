#![deny(missing_docs, missing_debug_implementations)]

//! Automation that starts where it stops.

pub use crate::train::Train;

pub mod fmt;
pub mod rt_logic;

mod train;

pub use choochoo_cfg_model as cfg_model;
pub use choochoo_rt_model as rt_model;

//! Configuration data types for the choochoo automation library.
//!
//! Data that encodes the train execution plan. Types in this module are
//! analogous to source artifacts.

pub use daggy;
pub use indexmap;
pub use indicatif;
pub use resman;
pub use rt_map;
pub use srcerr;

pub use crate::{
    setup_fn::{SetupFn, SetupFnReturn},
    station_fn::{StationFn, StationFnReturn},
    station_id::StationId,
    station_id_invalid_fmt::StationIdInvalidFmt,
    station_spec::StationSpec,
    station_spec_builder::StationSpecBuilder,
    station_spec_fns::StationSpecFns,
    station_specs::{StationSpecs, StationsFrozen},
    workload::Workload,
};

pub mod rt;

mod setup_fn;
mod station_fn;
mod station_id;
mod station_id_invalid_fmt;
mod station_spec;
mod station_spec_builder;
mod station_spec_fns;
mod station_specs;
mod workload;

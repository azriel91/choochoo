//! Configuration data types for the choochoo automation library.
//!
//! Data that encodes the train execution plan. Types in this module are
//! analogous to source artifacts.

pub use daggy;
pub use indicatif;
pub use resman;

pub use crate::{
    check_status::CheckStatus,
    progress_unit::ProgressUnit,
    station_fn::{StationFn, StationFnReturn},
    station_id::StationId,
    station_id_invalid_fmt::StationIdInvalidFmt,
    station_progress::StationProgress,
    station_spec::StationSpec,
    station_spec_builder::StationSpecBuilder,
    station_spec_fns::StationSpecFns,
    station_specs::{StationSpecs, StationsFrozen},
    visit_status::VisitStatus,
    workload::Workload,
};

mod check_status;
mod progress_unit;
mod station_fn;
mod station_id;
mod station_id_invalid_fmt;
mod station_progress;
mod station_spec;
mod station_spec_builder;
mod station_spec_fns;
mod station_specs;
mod visit_status;
mod workload;

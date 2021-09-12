//! Configuration data types for the choochoo automation library.
//!
//! Data that encodes the train execution plan. Types in this module are
//! analogous to source artifacts.

pub use indicatif;
pub use resman;

pub use crate::{
    check_status::CheckStatus,
    station_fn::{StationFn, StationFnReturn},
    station_id::StationId,
    station_id_invalid_fmt::StationIdInvalidFmt,
    station_progress::StationProgress,
    station_spec::StationSpec,
    station_spec_fns::StationSpecFns,
    visit_status::VisitStatus,
    workload::Workload,
};

mod check_status;
mod station_fn;
mod station_id;
mod station_id_invalid_fmt;
mod station_progress;
mod station_spec;
mod station_spec_fns;
mod visit_status;
mod workload;

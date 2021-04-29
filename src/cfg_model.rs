//! Data that encodes the train execution plan.
//!
//! Types in this module are analogous to source artifacts.

pub use self::{
    check_status::CheckStatus,
    station_fn::{StationFn, StationFnReturn},
    station_id::StationId,
    station_id_invalid_fmt::StationIdInvalidFmt,
    station_spec::StationSpec,
    station_spec_fns::StationSpecFns,
    workload::Workload,
};

mod check_status;
mod station_fn;
mod station_id;
mod station_id_invalid_fmt;
mod station_spec;
mod station_spec_fns;
mod workload;

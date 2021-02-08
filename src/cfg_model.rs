//! Data that encodes the train execution plan.
//!
//! Types in this module are analogous to source artifacts.

pub use self::{
    station_id::StationId,
    station_id_invalid_fmt::StationIdInvalidFmt,
    station_spec::StationSpec,
    visit_fn::{VisitFn, VisitFnReturn},
    workload::Workload,
};

mod station_id;
mod station_id_invalid_fmt;
mod station_spec;
mod visit_fn;
mod workload;

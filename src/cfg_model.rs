//! Data that encodes the train execution plan.
//!
//! Types in this module are analogous to source artifacts.

pub use self::{
    station_spec::StationSpec,
    visit_fn::{VisitFn, VisitFnReturn},
    workload::Workload,
};

mod station_spec;
mod visit_fn;
mod workload;

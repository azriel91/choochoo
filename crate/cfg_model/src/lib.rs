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
    check_status::CheckStatus,
    files::{Files, RwFiles},
    progress_limit::ProgressLimit,
    setup_fn::{SetupFn, SetupFnReturn},
    station::Station,
    station_errors::StationErrors,
    station_fn::{StationFn, StationFnReturn},
    station_id::StationId,
    station_id_invalid_fmt::StationIdInvalidFmt,
    station_mut::StationMut,
    station_progress::StationProgress,
    station_rt_id::StationRtId,
    station_spec::StationSpec,
    station_spec_builder::StationSpecBuilder,
    station_spec_fns::StationSpecFns,
    station_specs::{StationSpecs, StationsFrozen},
    train_report::TrainReport,
    visit_status::VisitStatus,
    workload::Workload,
};

mod check_status;
mod files;
mod progress_limit;
mod setup_fn;
mod station;
mod station_errors;
mod station_fn;
mod station_id;
mod station_id_invalid_fmt;
mod station_mut;
mod station_progress;
mod station_rt_id;
mod station_spec;
mod station_spec_builder;
mod station_spec_fns;
mod station_specs;
mod train_report;
mod visit_status;
mod workload;

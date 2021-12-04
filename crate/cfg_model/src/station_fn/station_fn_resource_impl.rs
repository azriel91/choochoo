use rt_map::BorrowFail;

use crate::{
    rt::{StationMut, TrainReport},
    StationFnResource, StationFnReturn,
};

// Unfortunately we have to `include!` instead of use a `#[path]` attribute.
// Pending: <https://github.com/rust-lang/rust/issues/48250>
include!(concat!(
    env!("OUT_DIR"),
    "/station_fn/station_fn_resource_impl.rs"
));

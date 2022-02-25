use resman::BorrowFail;

use futures::future::LocalBoxFuture;

use crate::{
    rt::{StationMutRef, TrainResources},
    StationFnRes, StationFnResource,
};

// Unfortunately we have to `include!` instead of use a `#[path]` attribute.
// Pending: <https://github.com/rust-lang/rust/issues/48250>
include!(concat!(
    env!("OUT_DIR"),
    "/station_fn/station_fn_res_impl.rs"
));

use std::marker::PhantomData;

use futures::future::LocalBoxFuture;
use resman::BorrowFail;

use crate::{
    rt::{StationMutRef, TrainReport},
    StationFnRes,
};

/// Function that gets its arguments / parameters from a `TrainReport`.
pub struct StationFnResource<Fun, R, RErr, E, Args> {
    /// The actual function.
    pub func: Fun,
    /// Marker.
    pub(crate) marker: PhantomData<(Fun, R, RErr, E, Args)>,
}

impl<Fun, R, RErr, E, Args> StationFnResource<Fun, R, RErr, E, Args> {
    /// Returns a new `StationFnResource`.
    pub fn new(func: Fun) -> Self {
        Self {
            func,
            marker: PhantomData,
        }
    }
}

impl<Fun, R, RErr, E> StationFnRes<R, RErr, E> for StationFnResource<Fun, R, RErr, E, ()>
where
    Fun: for<'f> Fn(&'f mut StationMutRef<'_, E>) -> LocalBoxFuture<'f, Result<R, RErr>>,
{
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        _train_report: &'f2 TrainReport<E>,
    ) -> LocalBoxFuture<'f2, Result<R, RErr>> {
        (self.func)(station)
    }

    fn try_call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        _train_report: &'f2 TrainReport<E>,
    ) -> Result<LocalBoxFuture<'f2, Result<R, RErr>>, BorrowFail> {
        Ok((self.func)(station))
    }
}

// Unfortunately we have to `include!` instead of use a `#[path]` attribute.
// Pending: <https://github.com/rust-lang/rust/issues/48250>
include!(concat!(
    env!("OUT_DIR"),
    "/station_fn/station_fn_resource.rs"
));

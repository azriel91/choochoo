use std::marker::PhantomData;

use futures::future::LocalBoxFuture;
use resman::BorrowFail;

use crate::{
    rt::{StationMutRef, TrainReport},
    StationFnRes,
};

/// Function that gets its arguments / parameters from a `TrainReport`.
pub struct StationFnResource<Fun, Ret, E, Args> {
    /// The actual function.
    pub func: Fun,
    /// Marker.
    pub(crate) marker: PhantomData<(Fun, Ret, E, Args)>,
}

impl<Fun, Ret, E, Args> StationFnResource<Fun, Ret, E, Args> {
    /// Returns a new `StationFnResource`.
    pub fn new(func: Fun) -> Self {
        Self {
            func,
            marker: PhantomData,
        }
    }
}

impl<Fun, Ret, E> StationFnRes<Ret, E> for StationFnResource<Fun, Ret, E, ()>
where
    Fun: for<'f> Fn(&'f mut StationMutRef<'_, E>) -> LocalBoxFuture<'f, Ret>,
{
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        _train_report: &'f2 TrainReport<E>,
    ) -> LocalBoxFuture<'f2, Ret> {
        (self.func)(station)
    }

    fn try_call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        _train_report: &'f2 TrainReport<E>,
    ) -> Result<LocalBoxFuture<'f2, Ret>, BorrowFail> {
        Ok((self.func)(station))
    }
}

// Unfortunately we have to `include!` instead of use a `#[path]` attribute.
// Pending: <https://github.com/rust-lang/rust/issues/48250>
include!(concat!(
    env!("OUT_DIR"),
    "/station_fn/station_fn_resource.rs"
));

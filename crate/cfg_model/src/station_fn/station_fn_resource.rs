use std::marker::PhantomData;

use crate::{
    rt::{StationMut, TrainReport},
    StationFnRes, StationFnReturn,
};

/// Function that gets its arguments / parameters from a `TrainReport`.
pub struct StationFnResource<Fun, R, E, Args> {
    /// The actual function.
    pub func: Fun,
    /// Marker.
    pub(crate) marker: PhantomData<(Fun, R, E, Args)>,
}

impl<Fun, R, E, Args> StationFnResource<Fun, R, E, Args> {
    /// Returns a new `StationFnResource`.
    pub fn new(func: Fun) -> Self {
        Self {
            func,
            marker: PhantomData,
        }
    }
}

impl<Fun, R, E> StationFnRes<R, E> for StationFnResource<Fun, R, E, ()>
where
    Fun: for<'f> Fn(&'f mut StationMut<'_, E>) -> StationFnReturn<'f, R, E>,
{
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMut<'_, E>,
        _train_report: &'f2 TrainReport<E>,
    ) -> StationFnReturn<'f2, R, E> {
        (self.func)(station)
    }
}

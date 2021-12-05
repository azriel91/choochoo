use std::marker::PhantomData;

use rt_map::BorrowFail;

use crate::{
    rt::{StationMut, TrainReport},
    StationFnReturn,
};

/// Function that gets its arguments / parameters from a `TrainReport`.
pub struct StationFnResource<Fun, R, E, Args> {
    /// The actual function.
    pub func: Fun,
    /// Marker.
    pub(crate) marker: PhantomData<(Fun, R, E, Args)>,
}

impl<Fun, R, E> StationFnResource<Fun, R, E, ()>
where
    Fun: for<'f> Fn(&'f mut StationMut<'_, E>) -> StationFnReturn<'f, R, E> + 'static,
{
    pub fn call<'f>(
        &self,
        station: &'f mut StationMut<'_, E>,
        _train_report: &TrainReport<E>,
    ) -> StationFnReturn<'f, R, E> {
        (self.func)(station)
    }

    pub fn try_call<'f>(
        &self,
        station: &'f mut StationMut<'_, E>,
        _train_report: &TrainReport<E>,
    ) -> Result<StationFnReturn<'f, R, E>, BorrowFail> {
        let ret_value = (self.func)(station);
        Ok(ret_value)
    }
}

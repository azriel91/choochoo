use rt_map::BorrowFail;

use crate::{
    rt::{StationMut, TrainReport},
    StationFnRes, StationFnResource, StationFnReturn,
};

impl<Fun, R, E> StationFnRes<R, E> for StationFnResource<Fun, R, E, ()>
where
    Fun: for<'f> Fn(&'f mut StationMut<'_, E>) -> StationFnReturn<'f, R, E> + 'static,
{
    fn call<'f>(
        &self,
        station: &'f mut StationMut<'_, E>,
        train_report: &TrainReport<E>,
    ) -> StationFnReturn<'f, R, E> {
        Self::call(self, station, train_report)
    }

    fn try_call<'f>(
        &self,
        station: &'f mut StationMut<'_, E>,
        train_report: &TrainReport<E>,
    ) -> Result<StationFnReturn<'f, R, E>, BorrowFail> {
        Self::try_call(self, station, train_report)
    }
}

// Unfortunately we have to `include!` instead of use a `#[path]` attribute.
// Pending: <https://github.com/rust-lang/rust/issues/48250>
include!(concat!(
    env!("OUT_DIR"),
    "/station_fn/station_fn_res_impl.rs"
));

use std::ops::Deref;

use resman::BorrowFail;

use crate::{
    rt::{StationMut, TrainReport},
    StationFnReturn,
};

/// Function that gets its arguments / parameters from a `TrainReport`.
///
/// This allows consumers of this library to hold onto multiple *resource
/// functions* as `Box<dyn StationFnRes<R, E>>`, even though their arguments
/// may be different.
pub trait StationFnRes<R, E> {
    /// Runs the function.
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMut<'_, E>,
        train_report: &'f2 TrainReport<E>,
    ) -> StationFnReturn<'f2, R, E>;

    fn try_call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMut<'_, E>,
        _train_report: &'f2 TrainReport<E>,
    ) -> Result<StationFnReturn<'f2, R, E>, BorrowFail>;
}

impl<Fun, R, E> StationFnRes<R, E> for Box<Fun>
where
    Fun: StationFnRes<R, E>,
{
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMut<'_, E>,
        train_report: &'f2 TrainReport<E>,
    ) -> StationFnReturn<'f2, R, E> {
        self.deref().call(station, train_report)
    }

    fn try_call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMut<'_, E>,
        train_report: &'f2 TrainReport<E>,
    ) -> Result<StationFnReturn<'f2, R, E>, BorrowFail> {
        self.deref().try_call(station, train_report)
    }
}

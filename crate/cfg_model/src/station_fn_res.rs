use std::ops::Deref;

use resman::BorrowFail;

use crate::{rt::TrainReport, StationFnReturn};

/// Function that gets its arguments / parameters from a `TrainReport`.
///
/// This allows consumers of this library to hold onto multiple *resource
/// functions* as `Box<dyn StationFnRes<'f, R, E>>`, even though their arguments
/// may be different.
pub trait StationFnRes<'f, R, E> {
    /// Runs the function.
    fn call(&self, train_report: &TrainReport<E>) -> StationFnReturn<'f, R, E>;

    /// Runs the function.
    fn try_call(
        &self,
        train_report: &TrainReport<E>,
    ) -> Result<StationFnReturn<'f, R, E>, BorrowFail>;
}

impl<'f, Fun, R, E> StationFnRes<'f, R, E> for Box<Fun>
where
    Fun: StationFnRes<'f, R, E>,
{
    fn call(&self, train_report: &TrainReport<E>) -> StationFnReturn<'f, R, E> {
        self.deref().call(train_report)
    }

    fn try_call(
        &self,
        train_report: &TrainReport<E>,
    ) -> Result<StationFnReturn<'f, R, E>, BorrowFail> {
        self.deref().try_call(train_report)
    }
}

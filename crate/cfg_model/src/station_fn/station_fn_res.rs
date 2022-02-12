use std::ops::Deref;

use futures::future::LocalBoxFuture;
use resman::BorrowFail;

use crate::rt::{StationMutRef, TrainReport};

/// Function that gets its arguments / parameters from a `TrainReport`.
///
/// This allows consumers of this library to hold onto multiple *resource
/// functions* as `Box<dyn StationFnRes<Ret, E>>`, even though their arguments
/// may be different.
pub trait StationFnRes<Ret, E> {
    /// Runs the function.
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        train_report: &'f2 TrainReport<E>,
    ) -> LocalBoxFuture<'f2, Ret>;

    fn try_call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        _train_report: &'f2 TrainReport<E>,
    ) -> Result<LocalBoxFuture<'f2, Ret>, BorrowFail>;
}

impl<Fun, Ret, E> StationFnRes<Ret, E> for Box<Fun>
where
    Fun: StationFnRes<Ret, E>,
{
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        train_report: &'f2 TrainReport<E>,
    ) -> LocalBoxFuture<'f2, Ret> {
        self.deref().call(station, train_report)
    }

    fn try_call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        train_report: &'f2 TrainReport<E>,
    ) -> Result<LocalBoxFuture<'f2, Ret>, BorrowFail> {
        self.deref().try_call(station, train_report)
    }
}

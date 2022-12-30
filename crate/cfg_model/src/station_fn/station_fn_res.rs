use std::ops::Deref;

use futures::future::LocalBoxFuture;
use resman::BorrowFail;

use crate::rt::{StationMutRef, TrainResources};

/// Function that gets its arguments / parameters from a `TrainResources`.
///
/// This allows consumers of this library to hold onto multiple *resource
/// functions* as `Box<dyn StationFnRes<R, E>>`, even though their arguments
/// may be different.
pub trait StationFnRes<R, RErr, E> {
    /// Runs the function.
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        train_resources: &'f2 TrainResources<E>,
    ) -> LocalBoxFuture<'f2, Result<R, RErr>>;

    fn try_call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        _train_resources: &'f2 TrainResources<E>,
    ) -> Result<LocalBoxFuture<'f2, Result<R, RErr>>, BorrowFail>;
}

impl<Fun, R, RErr, E> StationFnRes<R, RErr, E> for Box<Fun>
where
    Fun: StationFnRes<R, RErr, E>,
{
    fn call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        train_resources: &'f2 TrainResources<E>,
    ) -> LocalBoxFuture<'f2, Result<R, RErr>> {
        self.deref().call(station, train_resources)
    }

    fn try_call<'f1: 'f2, 'f2>(
        &'f2 self,
        station: &'f1 mut StationMutRef<'_, E>,
        train_resources: &'f2 TrainResources<E>,
    ) -> Result<LocalBoxFuture<'f2, Result<R, RErr>>, BorrowFail> {
        self.deref().try_call(station, train_resources)
    }
}

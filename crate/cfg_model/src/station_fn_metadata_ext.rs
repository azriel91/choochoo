use std::marker::PhantomData;

use fn_graph::FnMetadata;

use crate::{rt::StationMut, StationFnReturn};

/// Extension to return [`FnMetadata`] for a function.
pub trait StationFnMetadataExt<Fun, R, E, Args> {
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, Args>;
}

impl<Fun, R, E> StationFnMetadataExt<Fun, R, E, ()> for Fun
where
    Fun: for<'f> FnOnce(&'f mut StationMut<'_, E>) -> StationFnReturn<'f, R, E>,
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, ()> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A> StationFnMetadataExt<Fun, R, E, (A,)> for Fun
where
    Fun: for<'f> FnOnce(&'f mut StationMut<'_, E>, A) -> StationFnReturn<'f, R, E>,
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A,)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1> StationFnMetadataExt<Fun, R, E, (A0, A1)> for Fun
where
    Fun: for<'f> FnOnce(&'f mut StationMut<'_, E>, A0, A1) -> StationFnReturn<'f, R, E>,
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1, A2> StationFnMetadataExt<Fun, R, E, (A0, A1, A2)> for Fun
where
    Fun: for<'f> FnOnce(&'f mut StationMut<'_, E>, A0, A1, A2) -> StationFnReturn<'f, R, E>,
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1, A2)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1, A2, A3> StationFnMetadataExt<Fun, R, E, (A0, A1, A2, A3)> for Fun
where
    Fun: for<'f> FnOnce(&'f mut StationMut<'_, E>, A0, A1, A2, A3) -> StationFnReturn<'f, R, E>,
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1, A2, A3)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1, A2, A3, A4> StationFnMetadataExt<Fun, R, E, (A0, A1, A2, A3, A4)> for Fun
where
    Fun: for<'f> FnOnce(&'f mut StationMut<'_, E>, A0, A1, A2, A3, A4) -> StationFnReturn<'f, R, E>,
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1, A2, A3, A4)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1, A2, A3, A4, A5> StationFnMetadataExt<Fun, R, E, (A0, A1, A2, A3, A4, A5)>
    for Fun
where
    Fun: for<'f> FnOnce(
        &'f mut StationMut<'_, E>,
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
    ) -> StationFnReturn<'f, R, E>,
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1, A2, A3, A4, A5)> {
        FnMetadata(PhantomData)
    }
}

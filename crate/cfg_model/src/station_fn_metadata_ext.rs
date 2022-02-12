#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use fn_graph::FnMetadata;
use futures::future::LocalBoxFuture;

/// Extension to return [`FnMetadata`] for a function.
pub trait StationFnMetadataExt<Fun, Ret, E, Args> {
    fn metadata<'f>(&self) -> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, Args>;
}

impl<Fun, Ret, E> StationFnMetadataExt<Fun, Ret, E, ()> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, ()> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, Ret, E, A> StationFnMetadataExt<Fun, Ret, E, (A,)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (A,)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, Ret, E, A0, A1> StationFnMetadataExt<Fun, Ret, E, (A0, A1)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (A0, A1)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, Ret, E, A0, A1, A2> StationFnMetadataExt<Fun, Ret, E, (A0, A1, A2)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (A0, A1, A2)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, Ret, E, A0, A1, A2, A3> StationFnMetadataExt<Fun, Ret, E, (A0, A1, A2, A3)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (A0, A1, A2, A3)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, Ret, E, A0, A1, A2, A3, A4> StationFnMetadataExt<Fun, Ret, E, (A0, A1, A2, A3, A4)>
    for Fun
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (A0, A1, A2, A3, A4)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, Ret, E, A0, A1, A2, A3, A4, A5>
    StationFnMetadataExt<Fun, Ret, E, (A0, A1, A2, A3, A4, A5)> for Fun
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (A0, A1, A2, A3, A4, A5)> {
        FnMetadata(PhantomData)
    }
}

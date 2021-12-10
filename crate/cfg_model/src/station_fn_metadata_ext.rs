use std::marker::PhantomData;

use fn_graph::FnMetadata;

use crate::StationFnReturn;

/// Extension to return [`FnMetadata`] for a function.
pub trait StationFnMetadataExt<Fun, R, E, Args> {
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, Args>;
}

impl<Fun, R, E> StationFnMetadataExt<Fun, R, E, ()> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, ()> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A> StationFnMetadataExt<Fun, R, E, (A,)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A,)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1> StationFnMetadataExt<Fun, R, E, (A0, A1)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1, A2> StationFnMetadataExt<Fun, R, E, (A0, A1, A2)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1, A2)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1, A2, A3> StationFnMetadataExt<Fun, R, E, (A0, A1, A2, A3)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1, A2, A3)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1, A2, A3, A4> StationFnMetadataExt<Fun, R, E, (A0, A1, A2, A3, A4)> for Fun {
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1, A2, A3, A4)> {
        FnMetadata(PhantomData)
    }
}

impl<Fun, R, E, A0, A1, A2, A3, A4, A5> StationFnMetadataExt<Fun, R, E, (A0, A1, A2, A3, A4, A5)>
    for Fun
{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, (A0, A1, A2, A3, A4, A5)> {
        FnMetadata(PhantomData)
    }
}

#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use crate::StationFnResource;

/// Extension to return [`StationFnResource`] for a function.
pub trait IntoStationFnResource<Fun, R, E, Args> {
    /// Returns the function wrapped as a `StationFnResource`.
    fn into_station_fn_resource(self) -> StationFnResource<Fun, R, E, Args>;
}

impl<Fun, R, E> IntoStationFnResource<Fun, R, E, ()> for Fun {
    fn into_station_fn_resource(self) -> StationFnResource<Fun, R, E, ()> {
        StationFnResource {
            func: self,
            marker: PhantomData,
        }
    }
}

impl<Fun, R, E, A0> IntoStationFnResource<Fun, R, E, (A0,)> for Fun {
    fn into_station_fn_resource(self) -> StationFnResource<Fun, R, E, (A0,)> {
        StationFnResource {
            func: self,
            marker: PhantomData,
        }
    }
}

impl<Fun, R, E, A0, A1> IntoStationFnResource<Fun, R, E, (A0, A1)> for Fun {
    fn into_station_fn_resource(self) -> StationFnResource<Fun, R, E, (A0, A1)> {
        StationFnResource {
            func: self,
            marker: PhantomData,
        }
    }
}

impl<Fun, R, E, A0, A1, A2> IntoStationFnResource<Fun, R, E, (A0, A1, A2)> for Fun {
    fn into_station_fn_resource(self) -> StationFnResource<Fun, R, E, (A0, A1, A2)> {
        StationFnResource {
            func: self,
            marker: PhantomData,
        }
    }
}

impl<Fun, R, E, A0, A1, A2, A3> IntoStationFnResource<Fun, R, E, (A0, A1, A2, A3)> for Fun {
    fn into_station_fn_resource(self) -> StationFnResource<Fun, R, E, (A0, A1, A2, A3)> {
        StationFnResource {
            func: self,
            marker: PhantomData,
        }
    }
}

impl<Fun, R, E, A0, A1, A2, A3, A4> IntoStationFnResource<Fun, R, E, (A0, A1, A2, A3, A4)> for Fun {
    fn into_station_fn_resource(self) -> StationFnResource<Fun, R, E, (A0, A1, A2, A3, A4)> {
        StationFnResource {
            func: self,
            marker: PhantomData,
        }
    }
}

impl<Fun, R, E, A0, A1, A2, A3, A4, A5> IntoStationFnResource<Fun, R, E, (A0, A1, A2, A3, A4, A5)>
    for Fun
{
    fn into_station_fn_resource(self) -> StationFnResource<Fun, R, E, (A0, A1, A2, A3, A4, A5)> {
        StationFnResource {
            func: self,
            marker: PhantomData,
        }
    }
}

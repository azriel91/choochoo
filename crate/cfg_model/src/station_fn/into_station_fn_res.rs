use crate::{station_fn::IntoStationFnResource, StationFnRes, StationFnResource};

/// Extension to return `Box<dyn StationFnRes>` for a function.
pub trait IntoStationFnRes<Fun, R, E, Args> {
    /// R, Eurns the function wrapped as a `Box<dyn StationFnRes>`.
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E>>;
}

impl<Fun, R, E> IntoStationFnRes<Fun, R, E, ()> for Fun
where
    Fun: IntoStationFnResource<Fun, R, E, ()> + 'static,
    R: 'static,
    E: 'static,
    // Fun: for<'f> Fn(&'f mut StationMut<'_, E>) -> StationFnReturn<'f, R, E> + 'static,
    StationFnResource<Fun, R, E, ()>: StationFnRes<R, E>,
{
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E>> {
        Box::new(self.into_station_fn_resource())
    }
}

impl<Fun, R, E, A0> IntoStationFnRes<Fun, R, E, (A0,)> for Fun
where
    Fun: IntoStationFnResource<Fun, R, E, (A0,)> + 'static,
    R: 'static,
    E: 'static,
    // Fun: for<'f> Fn(&'f mut StationMut<'_, E>, A0) -> StationFnReturn<'f, R, E> + 'static,
    A0: 'static,
    StationFnResource<Fun, R, E, (A0,)>: StationFnRes<R, E>,
{
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E>> {
        Box::new(self.into_station_fn_resource())
    }
}

impl<Fun, R, E, A0, A1> IntoStationFnRes<Fun, R, E, (A0, A1)> for Fun
where
    Fun: IntoStationFnResource<Fun, R, E, (A0, A1)> + 'static,
    R: 'static,
    E: 'static,
    // Fun: for<'f> Fn(&'f mut StationMut<'_, E>, A0, A1) -> StationFnReturn<'f, R, E> + 'static,
    A0: 'static,
    A1: 'static,
    StationFnResource<Fun, R, E, (A0, A1)>: StationFnRes<R, E>,
{
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E>> {
        Box::new(self.into_station_fn_resource())
    }
}

impl<Fun, R, E, A0, A1, A2> IntoStationFnRes<Fun, R, E, (A0, A1, A2)> for Fun
where
    Fun: IntoStationFnResource<Fun, R, E, (A0, A1, A2)> + 'static,
    R: 'static,
    E: 'static,
    // Fun: for<'f> Fn(&'f mut StationMut<'_, E>, A0, A1, A2) -> StationFnReturn<'f, R, E> +
    // 'static,
    A0: 'static,
    A1: 'static,
    A2: 'static,
    StationFnResource<Fun, R, E, (A0, A1, A2)>: StationFnRes<R, E>,
{
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E>> {
        Box::new(self.into_station_fn_resource())
    }
}

impl<Fun, R, E, A0, A1, A2, A3> IntoStationFnRes<Fun, R, E, (A0, A1, A2, A3)> for Fun
where
    Fun: IntoStationFnResource<Fun, R, E, (A0, A1, A2, A3)> + 'static,
    R: 'static,
    E: 'static,
    // Fun: for<'f> Fn(&'f mut StationMut<'_, E>, A0, A1, A2, A3) -> StationFnReturn<'f, R, E> +
    // 'static,
    A0: 'static,
    A1: 'static,
    A2: 'static,
    A3: 'static,
    StationFnResource<Fun, R, E, (A0, A1, A2, A3)>: StationFnRes<R, E>,
{
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E>> {
        Box::new(self.into_station_fn_resource())
    }
}

impl<Fun, R, E, A0, A1, A2, A3, A4> IntoStationFnRes<Fun, R, E, (A0, A1, A2, A3, A4)> for Fun
where
    Fun: IntoStationFnResource<Fun, R, E, (A0, A1, A2, A3, A4)> + 'static,
    R: 'static,
    E: 'static,
    // Fun: for<'f> Fn(&'f mut StationMut<'_, E>, A0, A1, A2, A3, A4) -> StationFnReturn<'f, R, E>
    // + 'static,
    A0: 'static,
    A1: 'static,
    A2: 'static,
    A3: 'static,
    A4: 'static,
    StationFnResource<Fun, R, E, (A0, A1, A2, A3, A4)>: StationFnRes<R, E>,
{
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E>> {
        Box::new(self.into_station_fn_resource())
    }
}

impl<Fun, R, E, A0, A1, A2, A3, A4, A5> IntoStationFnRes<Fun, R, E, (A0, A1, A2, A3, A4, A5)>
    for Fun
where
    Fun: IntoStationFnResource<Fun, R, E, (A0, A1, A2, A3, A4, A5)> + 'static,
    R: 'static,
    E: 'static,
    // Fun: for<'f> Fn(&'f mut StationMut<'_, E>, A0, A1, A2, A3, A4, A5) -> StationFnReturn<'f, R,
    // E> + 'static,
    A0: 'static,
    A1: 'static,
    A2: 'static,
    A3: 'static,
    A4: 'static,
    A5: 'static,
    StationFnResource<Fun, R, E, (A0, A1, A2, A3, A4, A5)>: StationFnRes<R, E>,
{
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E>> {
        Box::new(self.into_station_fn_resource())
    }
}

use crate::{station_fn::IntoStationFnResource, StationFnRes, StationFnResource};

/// Extension to return `Box<dyn StationFnRes>` for a function.
pub trait IntoStationFnRes<'f, Fun, R, E, Args> {
    /// R, Eurns the function wrapped as a `Box<dyn StationFnRes>`.
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E> + 'f>;
}

impl<Fun, R, E> IntoStationFnRes<'static, Fun, R, E, ()> for Fun
where
    Fun: IntoStationFnResource<Fun, R, E, ()> + 'static,
    R: 'static,
    E: 'static,
    StationFnResource<Fun, R, E, ()>: StationFnRes<R, E>,
{
    fn into_station_fn_res(self) -> Box<dyn StationFnRes<R, E> + 'static> {
        Box::new(self.into_station_fn_resource())
    }
}

// Unfortunately we have to `include!` instead of use a `#[path]` attribute.
// Pending: <https://github.com/rust-lang/rust/issues/48250>
include!(concat!(
    env!("OUT_DIR"),
    "/station_fn/into_station_fn_res.rs"
));

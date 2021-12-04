use futures::future::LocalBoxFuture;

/// Return type of the `StationFn`.
pub type StationFnReturn<'f, R, E> = LocalBoxFuture<'f, Result<R, E>>;

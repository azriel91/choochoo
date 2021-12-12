use std::{future::Future, pin::Pin};

// use futures::future::LocalBoxFuture;

/// Return type of the `StationFn`.
pub type StationFnReturn<'f, R, E> = Pin<Box<dyn Future<Output = Result<R, E>> + 'f>>;
// pub type StationFnReturn<'f, R, E> = LocalBoxFuture<'f, Result<R, E>>;

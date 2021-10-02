use std::{
    fmt::{self, Debug},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use resman::Resources;

#[cfg(feature = "mock")]
use crate::VisitStatus;
use crate::{ProgressLimit, StationProgress};

/// Return type of the `SetupFn`.
pub type SetupFnReturn<'f, E> = Pin<Box<dyn Future<Output = Result<ProgressLimit, E>> + 'f>>;

// **Note:** `Debug`, `Clone`, `PartialEq` are manually implemented to avoid the
// trait bound on `E`.
/// Verifies input parameters, calculates progress limit, and inserts resources.
#[allow(clippy::type_complexity)] // trait aliases don't exist yet, so we have to suppress clippy.
pub struct SetupFn<E>(
    pub Arc<dyn for<'f> Fn(&'f mut StationProgress, &'f mut Resources) -> SetupFnReturn<'f, E>>,
);

impl<E> SetupFn<E> {
    /// Returns a new `SetupFn`.
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new<F>(f: F) -> Self
    where
        F: for<'f> Fn(&'f mut StationProgress, &'f mut Resources) -> SetupFnReturn<'f, E> + 'static,
    {
        Self(Arc::new(f))
    }

    /// Returns a `SetupFn` that always returns `Result::Ok`.
    #[cfg(feature = "mock")]
    pub fn ok(progress_limit: ProgressLimit) -> Self {
        SetupFn::new(move |_, _| {
            Box::pin(async move { Result::<ProgressLimit, E>::Ok(progress_limit) })
        })
    }

    /// Returns a `SetupFn` that always returns `Result::Err`.
    #[cfg(feature = "mock")]
    pub fn err(e: E) -> Self
    where
        E: Clone + 'static,
    {
        SetupFn::new(move |station_progress, _| {
            let e = e.clone();
            Box::pin(async move {
                station_progress.visit_status = VisitStatus::SetupFail;
                Result::<ProgressLimit, E>::Err(e)
            })
        })
    }
}

// We `impl Clone` to avoid the `E: Clone` bound generated by the derive.
#[cfg(not(tarpaulin_include))]
impl<E> Clone for SetupFn<E> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<E> Debug for SetupFn<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SetupFn(fn(&'_ mut Station<E>) -> SetupFnReturn<'_, E>)")
    }
}

impl<E> PartialEq for SetupFn<E> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.0, &other.0)
    }
}
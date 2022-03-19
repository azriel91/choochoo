use crate::OpFns;

/// Functions for cleaning an operation's resources.
pub type CleanFns<E> = OpFns<(), E, E>;

#[cfg(feature = "mock")]
impl<E> CleanFns<E>
where
    E: 'static,
{
    /// Returns new [`CleanFns`].
    ///
    /// * The [`setup_fn`] returns `Ok(ProgressLimit::Unknown)`.
    /// * The [`check_fn`] is defaulted to `None`.
    /// * The [`work_fn`] returns `Ok(())`.
    ///
    /// [`setup_fn`]: OpFns::setup_fn
    /// [`check_fn`]: OpFns::check_fn
    /// [`work_fn`]: OpFns::work_fn
    pub fn ok() -> CleanFns<E> {
        use crate::{rt::ProgressLimit, SetupFn, StationFn};

        let setup_fn = SetupFn::ok(ProgressLimit::Unknown);
        let work_fn = StationFn::ok(());
        Self::new(setup_fn, work_fn)
    }

    /// Returns new [`CleanFns`].
    ///
    /// * The [`setup_fn`] returns `Ok(ProgressLimit::Unknown)`.
    /// * The [`check_fn`] is defaulted to `None`.
    /// * The [`work_fn`] returns `Err(e)`.
    ///
    /// [`setup_fn`]: OpFns::setup_fn
    /// [`check_fn`]: OpFns::check_fn
    /// [`work_fn`]: OpFns::work_fn
    pub fn err(e: E) -> CleanFns<E>
    where
        E: Clone + 'static,
    {
        use crate::{rt::ProgressLimit, SetupFn, StationFn};

        let setup_fn = SetupFn::ok(ProgressLimit::Unknown);
        let work_fn = StationFn::err(e);
        Self::new(setup_fn, work_fn)
    }
}

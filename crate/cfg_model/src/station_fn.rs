use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use fn_graph::{FnMeta, FnMetadata, TypeIds};
use futures::future::{FutureExt, LocalBoxFuture};

#[cfg(feature = "mock")]
use crate::rt::StationMutRef;
use crate::StationFnMetadataExt;

pub use self::{
    into_station_fn_res::IntoStationFnRes, into_station_fn_resource::IntoStationFnResource,
    station_fn_res::StationFnRes, station_fn_resource::StationFnResource,
};

mod into_station_fn_res;
mod into_station_fn_resource;
mod station_fn_res;
mod station_fn_res_impl;
mod station_fn_resource;

// **Note:** `Debug`, `Clone`, `PartialEq` are manually implemented to avoid the
// trait bound on `E`.
/// Steps to run for this part of the station's logic.
#[allow(clippy::type_complexity)] // trait aliases don't exist yet, so we have to suppress clippy.
pub struct StationFn<R, RErr, E> {
    /// Function that gets its arguments / parameters from a `TrainResources`.
    ///
    /// This is wrapped in an [`Arc`] because we are unable to clone trait
    /// objects.
    pub f: Arc<Box<dyn StationFnRes<R, RErr, E>>>,
    /// [`TypeId`]s of borrowed arguments.
    ///
    /// [`TypeId`]: core::any::TypeId
    borrows: TypeIds,
    /// [`TypeId`]s of mutably borrowed arguments.
    ///
    /// [`TypeId`]: core::any::TypeId
    borrow_muts: TypeIds,
}

impl<R, RErr, E> StationFn<R, RErr, E>
where
    R: 'static,
    RErr: 'static,
    E: 'static,
{
    /// Returns a new `StationFn`.
    ///
    /// # Implementation Note
    ///
    /// We need the first `Fn` bound for Rust to apply the appropriate lifetime
    /// constraints to elided closure lifetimes.
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new<Fun, ArgRefs>(f: Fun) -> Self
    where
        Fun: IntoStationFnRes<Fun, R, RErr, E, ArgRefs>
            + StationFnMetadataExt<Fun, R, RErr, ArgRefs>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Result<R, RErr>>, ArgRefs>: FnMeta,
        ArgRefs: 'static,
    {
        let metadata = f.metadata();
        let f = f.into_station_fn_res();
        Self {
            f: Arc::new(f),
            borrows: metadata.borrows(),
            borrow_muts: metadata.borrow_muts(),
        }
    }

    /// Returns a new `StationFn`.
    ///
    /// This method allows you to construct a StationFn using a closure, as it
    /// places an appropriate lifetime constraint on the closure.
    ///
    /// We need the first `Fn` bound for Rust to apply the appropriate lifetime
    /// constraints to elided closure lifetimes.
    ///
    /// See:
    ///
    /// * <https://users.rust-lang.org/t/unhelpful-mismatched-types-error-message/48394>
    /// * <https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#10-closures-follow-the-same-lifetime-elision-rules-as-functions>
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new0<Fun>(f: Fun) -> Self
    where
        Fun: for<'f> Fn(&'f mut StationMutRef<'_, E>) -> LocalBoxFuture<'f, Result<R, RErr>>
            + IntoStationFnRes<Fun, R, RErr, E, ()>
            + StationFnMetadataExt<Fun, R, RErr, ()>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Result<R, RErr>>, ()>: FnMeta,
    {
        Self::new(f)
    }

    /// Returns a new `StationFn`.
    ///
    /// This method allows you to construct a StationFn using a closure, as it
    /// places an appropriate lifetime constraint on the closure.
    ///
    /// We need the first `Fn` bound for Rust to apply the appropriate lifetime
    /// constraints to elided closure lifetimes.
    ///
    /// See:
    ///
    /// * <https://users.rust-lang.org/t/unhelpful-mismatched-types-error-message/48394>
    /// * <https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#10-closures-follow-the-same-lifetime-elision-rules-as-functions>
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new1<Fun, A0>(f: Fun) -> Self
    where
        Fun: for<'f> Fn(&'f mut StationMutRef<'_, E>, &'f A0) -> LocalBoxFuture<'f, Result<R, RErr>>
            + IntoStationFnRes<Fun, R, RErr, E, (&'static A0,)>
            + for<'f> StationFnMetadataExt<Fun, R, RErr, (&'f A0,)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Result<R, RErr>>, (&'f A0,)>: FnMeta,
        A0: 'static,
    {
        Self::new(f)
    }

    /// Returns a new `StationFn`.
    ///
    /// This method allows you to construct a StationFn using a closure, as it
    /// places an appropriate lifetime constraint on the closure.
    ///
    /// We need the first `Fn` bound for Rust to apply the appropriate lifetime
    /// constraints to elided closure lifetimes.
    ///
    /// See:
    ///
    /// * <https://users.rust-lang.org/t/unhelpful-mismatched-types-error-message/48394>
    /// * <https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#10-closures-follow-the-same-lifetime-elision-rules-as-functions>
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new2<Fun, A0, A1>(f: Fun) -> Self
    where
        Fun: for<'f> Fn(
                &'f mut StationMutRef<'_, E>,
                &'f A0,
                &'f A1,
            ) -> LocalBoxFuture<'f, Result<R, RErr>>
            + IntoStationFnRes<Fun, R, RErr, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<Fun, R, RErr, (&'f A0, &'f A1)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Result<R, RErr>>, (&'f A0, &'f A1)>: FnMeta,
        A0: 'static,
        A1: 'static,
    {
        Self::new(f)
    }

    /// Returns a new `StationFn`.
    ///
    /// This method allows you to construct a StationFn using a closure, as it
    /// places an appropriate lifetime constraint on the closure.
    ///
    /// We need the first `Fn` bound for Rust to apply the appropriate lifetime
    /// constraints to elided closure lifetimes.
    ///
    /// See:
    ///
    /// * <https://users.rust-lang.org/t/unhelpful-mismatched-types-error-message/48394>
    /// * <https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#10-closures-follow-the-same-lifetime-elision-rules-as-functions>
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new3<Fun, A0, A1, A2>(f: Fun) -> Self
    where
        Fun: for<'f> Fn(
                &'f mut StationMutRef<'_, E>,
                &'f A0,
                &'f A1,
                &'f A2,
            ) -> LocalBoxFuture<'f, Result<R, RErr>>
            + IntoStationFnRes<Fun, R, RErr, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<Fun, R, RErr, (&'f A0, &'f A1, &'f A2)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Result<R, RErr>>, (&'f A0, &'f A1, &'f A2)>:
            FnMeta,
        A0: 'static,
        A1: 'static,
        A2: 'static,
    {
        Self::new(f)
    }

    /// Returns a new `StationFn`.
    ///
    /// This method allows you to construct a StationFn using a closure, as it
    /// places an appropriate lifetime constraint on the closure.
    ///
    /// We need the first `Fn` bound for Rust to apply the appropriate lifetime
    /// constraints to elided closure lifetimes.
    ///
    /// See:
    ///
    /// * <https://users.rust-lang.org/t/unhelpful-mismatched-types-error-message/48394>
    /// * <https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#10-closures-follow-the-same-lifetime-elision-rules-as-functions>
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new4<Fun, A0, A1, A2, A3>(f: Fun) -> Self
    where
        Fun: for<'f> Fn(
                &'f mut StationMutRef<'_, E>,
                &'f A0,
                &'f A1,
                &'f A2,
                &'f A3,
            ) -> LocalBoxFuture<'f, Result<R, RErr>>
            + IntoStationFnRes<Fun, R, RErr, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<Fun, R, RErr, (&'f A0, &'f A1, &'f A2, &'f A3)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Result<R, RErr>>, (&'f A0, &'f A1, &'f A2, &'f A3)>:
            FnMeta,
        A0: 'static,
        A1: 'static,
        A2: 'static,
        A3: 'static,
    {
        Self::new(f)
    }

    /// Returns a new `StationFn`.
    ///
    /// This method allows you to construct a StationFn using a closure, as it
    /// places an appropriate lifetime constraint on the closure.
    ///
    /// We need the first `Fn` bound for Rust to apply the appropriate lifetime
    /// constraints to elided closure lifetimes.
    ///
    /// See:
    ///
    /// * <https://users.rust-lang.org/t/unhelpful-mismatched-types-error-message/48394>
    /// * <https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#10-closures-follow-the-same-lifetime-elision-rules-as-functions>
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new5<Fun, A0, A1, A2, A3, A4>(f: Fun) -> Self
    where
        Fun: for<'f> Fn(
                &'f mut StationMutRef<'_, E>,
                &'f A0,
                &'f A1,
                &'f A2,
                &'f A3,
                &'f A4,
            ) -> LocalBoxFuture<'f, Result<R, RErr>>
            + IntoStationFnRes<Fun, R, RErr, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<Fun, R, RErr, (&'f A0, &'f A1, &'f A2, &'f A3, &'f A4)>
            + 'static,
        for<'f> FnMetadata<
            Fun,
            LocalBoxFuture<'f, Result<R, RErr>>,
            (&'f A0, &'f A1, &'f A2, &'f A3, &'f A4),
        >: FnMeta,
        A0: 'static,
        A1: 'static,
        A2: 'static,
        A3: 'static,
        A4: 'static,
    {
        Self::new(f)
    }

    /// Returns a new `StationFn`.
    ///
    /// This method allows you to construct a StationFn using a closure, as it
    /// places an appropriate lifetime constraint on the closure.
    ///
    /// We need the first `Fn` bound for Rust to apply the appropriate lifetime
    /// constraints to elided closure lifetimes.
    ///
    /// See:
    ///
    /// * <https://users.rust-lang.org/t/unhelpful-mismatched-types-error-message/48394>
    /// * <https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#10-closures-follow-the-same-lifetime-elision-rules-as-functions>
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new6<Fun, A0, A1, A2, A3, A4, A5>(f: Fun) -> Self
    where
        Fun: for<'f> Fn(
                &'f mut StationMutRef<'_, E>,
                &'f A0,
                &'f A1,
                &'f A2,
                &'f A3,
                &'f A4,
                &'f A5,
            ) -> LocalBoxFuture<'f, Result<R, RErr>>
            + IntoStationFnRes<Fun, R, RErr, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<
                Fun,
                R,
                E,
                (&'f A0, &'f A1, &'f A2, &'f A3, &'f A4, &'f A5),
            > + 'static,
        for<'f> FnMetadata<
            Fun,
            LocalBoxFuture<'f, Result<R, RErr>>,
            (&'f A0, &'f A1, &'f A2, &'f A3, &'f A4, &'f A5),
        >: FnMeta,
        A0: 'static,
        A1: 'static,
        A2: 'static,
        A3: 'static,
        A4: 'static,
        A5: 'static,
    {
        Self::new(f)
    }

    /// Returns a `StationFn` that always returns `Result::Ok`.
    #[cfg(feature = "mock")]
    pub fn ok(r: R) -> Self
    where
        R: Clone + 'static,
    {
        StationFn::new0(move |_: &mut StationMutRef<'_, E>| {
            let r = r.clone();
            async move { Result::<R, RErr>::Ok(r) }.boxed_local()
        })
    }

    /// Returns a `StationFn` that always returns `Result::Err`.
    #[cfg(feature = "mock")]
    pub fn err(e: RErr) -> Self
    where
        RErr: Clone + 'static,
    {
        StationFn::new0(move |_: &mut StationMutRef<'_, E>| {
            let e = e.clone();
            async move { Result::<R, RErr>::Err(e) }.boxed_local()
        })
    }
}

// We `impl Clone` to avoid the `E: Clone` bound generated by the derive.
#[cfg(not(tarpaulin_include))]
impl<R, RErr, E> Clone for StationFn<R, RErr, E> {
    fn clone(&self) -> Self {
        Self {
            f: Arc::clone(&self.f),
            borrows: self.borrows.clone(),
            borrow_muts: self.borrow_muts.clone(),
        }
    }
}

impl<R, RErr, E> Debug for StationFn<R, RErr, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            "StationFn(fn(&'_ mut Station<R, RErr, E>) -> LocalBoxFuture<'_, Result<R, RErr>>)",
        )
    }
}

impl<R, RErr, E> PartialEq for StationFn<R, RErr, E> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.f, &other.f)
    }
}

impl<R, RErr, E> FnMeta for StationFn<R, RErr, E> {
    fn borrows(&self) -> TypeIds {
        self.borrows.clone()
    }

    fn borrow_muts(&self) -> TypeIds {
        self.borrow_muts.clone()
    }
}

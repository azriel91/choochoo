use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use fn_graph::{FnMeta, FnMetadata, TypeIds};
use futures::future::{FutureExt, LocalBoxFuture};

#[cfg(feature = "mock")]
use crate::rt::{ResourceIds, StationMutRef};
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
pub struct StationFn<Ret, E> {
    /// Function that gets its arguments / parameters from a `TrainReport`.
    ///
    /// This is wrapped in an [`Arc`] because we are unable to clone trait
    /// objects.
    pub f: Arc<Box<dyn StationFnRes<Ret, E>>>,
    /// [`TypeId`]s of borrowed arguments.
    ///
    /// [`TypeId`]: core::any::TypeId
    borrows: TypeIds,
    /// [`TypeId`]s of mutably borrowed arguments.
    ///
    /// [`TypeId`]: core::any::TypeId
    borrow_muts: TypeIds,
}

impl<Ret, E> StationFn<Ret, E>
where
    Ret: 'static,
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
        Fun: IntoStationFnRes<Fun, Ret, E, ArgRefs>
            + StationFnMetadataExt<Fun, Ret, E, ArgRefs>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, ArgRefs>: FnMeta,
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
        Fun: for<'f> Fn(&'f mut StationMutRef<'_, E>) -> LocalBoxFuture<'f, Ret>
            + IntoStationFnRes<Fun, Ret, E, ()>
            + StationFnMetadataExt<Fun, Ret, E, ()>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, ()>: FnMeta,
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
        Fun: for<'f> Fn(&'f mut StationMutRef<'_, E>, &'f A0) -> LocalBoxFuture<'f, Ret>
            + IntoStationFnRes<Fun, Ret, E, (&'static A0,)>
            + for<'f> StationFnMetadataExt<Fun, Ret, E, (&'f A0,)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (&'f A0,)>: FnMeta,
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
        Fun: for<'f> Fn(&'f mut StationMutRef<'_, E>, &'f A0, &'f A1) -> LocalBoxFuture<'f, Ret>
            + IntoStationFnRes<Fun, Ret, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<Fun, Ret, E, (&'f A0, &'f A1)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (&'f A0, &'f A1)>: FnMeta,
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
            ) -> LocalBoxFuture<'f, Ret>
            + IntoStationFnRes<Fun, Ret, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<Fun, Ret, E, (&'f A0, &'f A1, &'f A2)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (&'f A0, &'f A1, &'f A2)>: FnMeta,
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
            ) -> LocalBoxFuture<'f, Ret>
            + IntoStationFnRes<Fun, Ret, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<Fun, Ret, E, (&'f A0, &'f A1, &'f A2, &'f A3)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (&'f A0, &'f A1, &'f A2, &'f A3)>: FnMeta,
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
            ) -> LocalBoxFuture<'f, Ret>
            + IntoStationFnRes<Fun, Ret, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<Fun, Ret, E, (&'f A0, &'f A1, &'f A2, &'f A3, &'f A4)>
            + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (&'f A0, &'f A1, &'f A2, &'f A3, &'f A4)>:
            FnMeta,
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
            ) -> LocalBoxFuture<'f, Ret>
            + IntoStationFnRes<Fun, Ret, E, (&'static A0, &'static A1)>
            + for<'f> StationFnMetadataExt<
                Fun,
                Ret,
                E,
                (&'f A0, &'f A1, &'f A2, &'f A3, &'f A4, &'f A5),
            > + 'static,
        for<'f> FnMetadata<Fun, LocalBoxFuture<'f, Ret>, (&'f A0, &'f A1, &'f A2, &'f A3, &'f A4, &'f A5)>:
            FnMeta,
        A0: 'static,
        A1: 'static,
        A2: 'static,
        A3: 'static,
        A4: 'static,
        A5: 'static,
    {
        Self::new(f)
    }
}

#[cfg(feature = "mock")]
impl<R, E> StationFn<Result<R, E>, E>
where
    R: 'static,
    E: 'static,
{
    /// Returns a `StationFn` that always returns `Result::Ok`.
    pub fn ok(r: R) -> Self
    where
        R: Clone + 'static,
    {
        StationFn::new0(move |_: &mut StationMutRef<'_, E>| {
            let r = r.clone();
            async move { Result::<R, E>::Ok(r) }.boxed_local()
        })
    }

    /// Returns a `StationFn` that always returns `Result::Err`.
    pub fn err(e: E) -> Self
    where
        E: Clone + 'static,
    {
        StationFn::new0(move |_: &mut StationMutRef<'_, E>| {
            let e = e.clone();
            async move { Result::<R, E>::Err(e) }.boxed_local()
        })
    }
}

#[cfg(feature = "mock")]
impl<E> StationFn<(ResourceIds, Result<(), E>), E>
where
    E: 'static,
{
    /// Returns a `StationFn` that always returns `Result::Ok`.
    pub fn create_ok(resource_ids: ResourceIds) -> Self {
        StationFn::new0(move |_: &mut StationMutRef<'_, E>| {
            let resource_ids = resource_ids.clone();
            async move { (resource_ids, Result::<(), E>::Ok(())) }.boxed_local()
        })
    }

    /// Returns a `StationFn` that always returns `Result::Err`.
    pub fn create_err(resource_ids: ResourceIds, e: E) -> Self
    where
        E: Clone + 'static,
    {
        StationFn::new0(move |_: &mut StationMutRef<'_, E>| {
            let resource_ids = resource_ids.clone();
            let e = e.clone();
            async move { (resource_ids, Result::<(), E>::Err(e)) }.boxed_local()
        })
    }
}

// We `impl Clone` to avoid the `E: Clone` bound generated by the derive.
#[cfg(not(tarpaulin_include))]
impl<Ret, E> Clone for StationFn<Ret, E> {
    fn clone(&self) -> Self {
        Self {
            f: Arc::clone(&self.f),
            borrows: self.borrows.clone(),
            borrow_muts: self.borrow_muts.clone(),
        }
    }
}

impl<Ret, E> Debug for StationFn<Ret, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("StationFn(fn(&'_ mut Station<'_, E>) -> LocalBoxFuture<'_, Ret>)")
    }
}

impl<Ret, E> PartialEq for StationFn<Ret, E> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.f, &other.f)
    }
}

impl<Ret, E> FnMeta for StationFn<Ret, E> {
    fn borrows(&self) -> TypeIds {
        self.borrows.clone()
    }

    fn borrow_muts(&self) -> TypeIds {
        self.borrow_muts.clone()
    }
}

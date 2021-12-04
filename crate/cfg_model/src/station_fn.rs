use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use fn_graph::{FnMeta, FnMetadata, TypeIds};

#[cfg(feature = "mock")]
use crate::rt::{StationMut, VisitStatus};
use crate::{StationFnMetadataExt, StationFnRes, StationFnReturn};

// **Note:** `Debug`, `Clone`, `PartialEq` are manually implemented to avoid the
// trait bound on `E`.
/// Steps to run for this part of the station's logic.
#[allow(clippy::type_complexity)] // trait aliases don't exist yet, so we have to suppress clippy.
pub struct StationFn<R, E> {
    ///
    pub f: Arc<dyn for<'f> StationFnRes<'f, R, E>>,
    /// [`TypeId`]s of borrowed arguments.
    ///
    /// [`TypeId`]: core::any::TypeId
    borrows: TypeIds,
    /// [`TypeId`]s of mutably borrowed arguments.
    ///
    /// [`TypeId`]: core::any::TypeId
    borrow_muts: TypeIds,
}

impl<R, E> StationFn<R, E> {
    /// Returns a new `StationFn`.
    ///
    /// # Parameters
    ///
    /// * `f`: Logic to run.
    pub fn new<Fun, Args>(f: Fun) -> Self
    where
        Fun: StationFnMetadataExt<Fun, R, E, Args> + for<'f> StationFnRes<'f, R, E> + 'static,
        // for<'f> FnMetadata<Fun, StationFnReturn<'f, R, E>, Args>: FnMeta,
    {
        // let metadata = f.metadata();
        Self {
            f: Arc::new(f),
            borrows: TypeIds::new(),
            borrow_muts: TypeIds::new(),
            /* borrows: metadata.borrows(),
             * borrow_muts: metadata.borrow_muts(), */
        }
    }

    /// Returns a `StationFn` that always returns `Result::Ok`.
    #[cfg(feature = "mock")]
    pub fn ok(r: R) -> Self
    where
        R: Clone + 'static,
    {
        StationFn::new(move |station: &mut StationMut<'_, E>| {
            let r = r.clone();
            Box::pin(async move {
                station.progress.visit_status = VisitStatus::VisitSuccess;
                Result::<R, E>::Ok(r)
            })
        })
    }

    // /// Returns a `StationFn` that always returns `Result::Err`.
    // #[cfg(feature = "mock")]
    // pub fn err(e: E) -> Self
    // where
    //     E: Clone + 'static,
    // {
    //     StationFn::new(move |station: &mut StationMut<'_, E>| {
    //         let e = e.clone();
    //         Box::pin(async move {
    //             station.progress.visit_status = VisitStatus::VisitFail;
    //             Result::<R, E>::Err(e)
    //         })
    //     })
    // }
}

// We `impl Clone` to avoid the `E: Clone` bound generated by the derive.
#[cfg(not(tarpaulin_include))]
impl<R, E> Clone for StationFn<R, E> {
    fn clone(&self) -> Self {
        Self {
            f: Arc::clone(&self.f),
            borrows: self.borrows.clone(),
            borrow_muts: self.borrow_muts.clone(),
        }
    }
}

impl<R, E> Debug for StationFn<R, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("StationFn(fn(&'_ mut Station<R, E>) -> StationFnReturn<'_, E>)")
    }
}

impl<R, E> PartialEq for StationFn<R, E> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.f, &other.f)
    }
}

impl<R, E> FnMeta for StationFn<R, E> {
    fn borrows(&self) -> TypeIds {
        self.borrows.clone()
    }

    fn borrow_muts(&self) -> TypeIds {
        self.borrow_muts.clone()
    }
}

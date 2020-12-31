use std::marker::PhantomData;

use crate::rt_model::Station;

/// Updates a station's [`VisitStatus`].
#[derive(Debug)]
pub struct VisitStatusUpdater<E> {
    /// Marker
    pub marker: PhantomData<E>,
}

impl<E> VisitStatusUpdater<E> {
    /// Updates a station's [`VisitStatus`].
    ///
    /// The new visit status is calculated based on the station's visit result
    /// and its parents' [`VisitStatus`]es.
    ///
    /// # `VisitStatus` State Machine
    ///
    /// ## `NotReady` Stations
    ///
    /// * If all parents are `VisitSuccess`, switch to `Ready`.
    /// * If at least one parent has `VisitFailed` or `ParentFail`, switch to
    ///   `ParentFail`.
    ///
    /// ## `ParentFail` Stations
    ///
    /// * If any parents are not in either `ParentFail` or `VisitFailed`, return
    ///   error.
    ///
    /// ## `Queued` Stations
    ///
    /// * If any parents are `NotReady`, `Queued`, or `InProgress`, return
    ///   error.
    /// * If at least one parent has `VisitFailed` or `ParentFail`, switch to
    ///   `ParentFail`.
    ///
    /// ## `InProgress` Stations
    ///
    /// * If `visit` is successful, switch to `VisitSuccess`.
    /// * If `visit` fails, switch to `VisitFailed`.
    ///
    /// ## `VisitSuccess`
    ///
    /// No transitions.
    ///
    /// ## `VisitFailed`
    ///
    /// No transitions.
    pub fn update(station: &mut Station<E>) {
        unimplemented!()
    }
}

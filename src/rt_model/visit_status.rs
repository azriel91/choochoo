/// Status of whether a [`Station`] has been visited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisitStatus {
    /// Station has at least one parent that hasn't been visited.
    NotReady,
    /// At least one of this station's parents failed to be visited.
    ///
    /// There will not be an attempt to visit this station.
    ParentFail,
    /// Station is ready to be visited, but has not been.
    Queued,
    /// Station check function failed.
    CheckFail,
    /// Station visit is in progress.
    ///
    /// There is a train at this station.
    InProgress,
    /// This station was not necessary to visit.
    VisitUnnecessary,
    /// This station has been successfully visited.
    VisitSuccess,
    /// This station has been visited, but the visit failed.
    VisitFail,
}

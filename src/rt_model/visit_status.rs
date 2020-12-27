/// Status of whether a [`Station`] has been visited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisitStatus {
    /// Station is not ready to be visited.
    ///
    /// Typically means other stations must be visited beforehand -- there is a
    /// dependency.
    NotReady,
    /// Station is ready to be visited, but has not been.
    Queued,
    /// Station visit is in progress.
    ///
    /// There is a train at this station.
    InProgress,
    /// This station has been successfully visited.
    VisitSuccess,
    /// This station has been visited, but the visit failed.
    VisitFail,
}

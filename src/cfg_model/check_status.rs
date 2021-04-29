/// Report of checking the status of a station.
///
/// # Development Note
///
/// This does not parameterize the station return type, as [`StationSpecFns`]
/// must not be type parameterized in order to be stored as the same node type
/// in the station graph.
///
/// Instead, the type that is checked should be read from the application's
/// [`Resources`].
///
/// [`Resources`]: resman::Resources
/// [`StationSpecFns`]: crate::cfg_model::StationSpecFns
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CheckStatus {
    /// Station is not in desired state.
    VisitRequired,
    /// Station is already in desired state.
    VisitNotRequired,
}

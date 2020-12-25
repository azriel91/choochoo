use crate::Stations;

/// Specification of a desired state.
pub trait Destination {
    /// Returns the stations along the way to the destination.
    fn stations(&self) -> &Stations;
    /// Returns the stations along the way to the destination.
    fn stations_mut(&mut self) -> &mut Stations;
}

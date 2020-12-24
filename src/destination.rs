use crate::Stations;

/// Specification of a desired state.
pub trait Destination {
    /// Returns whether this status is reached.
    fn is_reached(&self) -> bool;

    /// Stations along the way to the destination.
    fn stations(&mut self) -> &mut Stations;
}

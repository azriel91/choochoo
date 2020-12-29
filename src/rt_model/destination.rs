use crate::rt_model::Stations;

/// Specification of a desired state.
pub trait Destination<E> {
    /// Returns the stations along the way to the destination.
    fn stations(&self) -> &Stations<E>;
    /// Returns the stations along the way to the destination.
    fn stations_mut(&mut self) -> &mut Stations<E>;
}

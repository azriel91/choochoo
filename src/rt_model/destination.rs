use crate::rt_model::Stations;

/// Specification of a desired state.
#[derive(Clone, Debug, Default)]
pub struct Destination<E> {
    /// The stations along the way to the destination.
    pub stations: Stations<E>,
}

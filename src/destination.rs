/// Specification of a desired state.
pub trait Destination {
    /// Returns whether this status is reached.
    fn is_reached(&self) -> bool;
}

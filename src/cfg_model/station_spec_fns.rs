use crate::cfg_model::StationFn;

/// Grouping of a station's behaviours.
#[derive(Debug, Clone, PartialEq)]
pub struct StationSpecFns<E> {
    /// Steps to execute when visiting a station.
    pub visit_fn: StationFn<(), E>,
}

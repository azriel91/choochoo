use crate::rt_model::Station;

/// Record of what happened during a train's drive.
#[derive(Debug)]
pub struct TrainReport<'rt, E> {
    /// Stations successfully visited.
    pub stations_successful: Vec<&'rt Station<E>>,
    /// Stations that were visited but failed to work.
    pub stations_failed: Vec<&'rt Station<E>>,
}

impl<'rt, E> Default for TrainReport<'rt, E> {
    fn default() -> Self {
        Self {
            stations_successful: Default::default(),
            stations_failed: Default::default(),
        }
    }
}

impl<'rt, E> TrainReport<'rt, E> {
    /// Returns a new TrainReport.
    pub fn new() -> Self {
        Self::default()
    }
}

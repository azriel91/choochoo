use crate::rt_model::Station;

/// Record of what happened during a train's drive.
#[derive(Debug, Default)]
pub struct TrainReport<'rt> {
    /// Stations successfully visited.
    pub stations_successful: Vec<&'rt Station>,
}

impl<'rt> TrainReport<'rt> {
    /// Returns a new TrainReport.
    pub fn new() -> Self {
        Self::default()
    }
}

use rt_map::RefMut;

use crate::{
    rt::{ProgressLimit, StationDir, StationProgress, StationRtId, TrainReport},
    StationSpec,
};

/// Station runtime information.
///
/// This includes a mutable reference to the station's progress, so it can only
/// be constructed when nothing else has a reference to this station's progress.
#[derive(Debug)]
pub struct StationMut<'s, E> {
    /// Behaviour specification of the station.
    pub spec: &'s StationSpec<E>,
    /// Runtime identifier for a station.
    pub rt_id: StationRtId,
    /// Directory to hold data specific to each station.
    pub dir: &'s StationDir,
    /// Station progress to reaching the destination.
    pub progress: RefMut<'s, StationProgress>,
}

impl<'s, E> StationMut<'s, E> {
    /// Verifies input, calculates progress limit, and inserts resources.
    pub async fn setup(&mut self, train_report: &mut TrainReport<E>) -> Result<ProgressLimit, E> {
        let setup_fn = self.spec.station_op.create_op_fns().setup_fn.clone();
        setup_fn.0(self, train_report).await
    }
}

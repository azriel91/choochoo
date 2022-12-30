use rt_map::RefMut;

use crate::{
    rt::{ProgressLimit, StationDir, StationProgress, StationRtId, TrainResources},
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
    pub async fn create_setup(
        &mut self,
        train_resources: &mut TrainResources<E>,
    ) -> Result<ProgressLimit, E> {
        let setup_fn = self.spec.station_op.create_fns().setup_fn.clone();
        setup_fn.0(self, train_resources).await
    }

    /// Verifies input and inserts resources.
    pub async fn clean_setup(
        &mut self,
        train_resources: &mut TrainResources<E>,
    ) -> Option<Result<ProgressLimit, E>> {
        if let Some(clean_fns) = self.spec.station_op.clean_fns().as_ref() {
            let setup_fn = clean_fns.setup_fn.clone();
            Some(setup_fn.0(self, train_resources).await)
        } else {
            None
        }
    }
}

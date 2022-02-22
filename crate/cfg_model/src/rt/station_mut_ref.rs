use fn_graph::FnRef;
use rt_map::{BorrowFail, RefMut};

use crate::{
    rt::{CheckStatus, ResourceIds, StationDir, StationProgress, StationRtId, TrainReport},
    StationSpec,
};

/// Station runtime information.
///
/// This differs from [`StationMut`] by holding a [`FnRef`] to the spec, which
/// is needed when streaming stations concurrently.
///
/// This includes a mutable reference to the station's progress, so it can only
/// be constructed when nothing else has a reference to this station's progress.
#[derive(Debug)]
pub struct StationMutRef<'s, E> {
    /// Behaviour specification of the station.
    pub spec: FnRef<'s, StationSpec<E>>,
    /// Runtime identifier for a station.
    pub rt_id: StationRtId,
    /// Directory to hold data specific to each station.
    pub dir: &'s StationDir,
    /// Station progress to reaching the destination.
    pub progress: RefMut<'s, StationProgress>,
}

impl<'s, E> StationMutRef<'s, E>
where
    E: 'static,
{
    /// Checks if the station needs to be visited.
    pub async fn check<'f>(
        &'f mut self,
        train_report: &'f TrainReport<E>,
    ) -> Option<Result<Result<CheckStatus, E>, BorrowFail>> {
        let check_fn = self.spec.station_op.create_fns().check_fn.clone();
        if let Some(check_fn) = check_fn {
            let call = check_fn.f.try_call(self, train_report);
            match call {
                Ok(fut) => Some(Ok(fut.await)),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }

    /// Visits the station.
    pub async fn visit<'f>(
        &'f mut self,
        train_report: &'f TrainReport<E>,
    ) -> Result<Result<ResourceIds, (ResourceIds, E)>, BorrowFail> {
        let work_fn = self.spec.station_op.create_fns().work_fn.clone();
        let call = work_fn.f.try_call(self, train_report);
        match call {
            Ok(fut) => Ok(fut.await),
            Err(e) => Err(e),
        }
    }
}

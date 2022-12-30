use fn_graph::FnRef;
use rt_map::{BorrowFail, RefMut};

use crate::{
    rt::{CheckStatus, ResIds, StationDir, StationProgress, StationRtId, TrainResources},
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
    /// Checks if the create function needs to be run.
    pub async fn create_check<'f>(
        &'f mut self,
        train_resources: &'f TrainResources<E>,
    ) -> Option<Result<Result<CheckStatus, E>, BorrowFail>> {
        let check_fn = self.spec.station_op.create_fns().check_fn.clone();
        if let Some(check_fn) = check_fn {
            let call = check_fn.f.try_call(self, train_resources);
            match call {
                Ok(fut) => Some(Ok(fut.await)),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }

    /// Runs the create function.
    pub async fn create_visit<'f>(
        &'f mut self,
        train_resources: &'f TrainResources<E>,
    ) -> Result<Result<ResIds, (ResIds, E)>, BorrowFail> {
        let work_fn = self.spec.station_op.create_fns().work_fn.clone();
        let call = work_fn.f.try_call(self, train_resources);
        match call {
            Ok(fut) => Ok(fut.await),
            Err(e) => Err(e),
        }
    }

    /// Checks if the create function needs to be run.
    ///
    /// Layers:
    ///
    /// * First `Option` is whether the station supports cleaning.
    /// * Second `Option` is whether the station's clean function supports
    ///   checking.
    /// * Outer `Result` is whether the resources needed to clean are borrowed
    ///   successfully.
    /// * Inner `Result` is whether the clean function returned successfully.
    pub async fn clean_check<'f>(
        &'f mut self,
        train_resources: &'f TrainResources<E>,
    ) -> Option<Option<Result<Result<CheckStatus, E>, BorrowFail>>> {
        let clean_fns = self.spec.station_op.clean_fns();
        if let Some(clean_fns) = clean_fns {
            if let Some(check_fn) = clean_fns.check_fn.clone() {
                let call = check_fn.f.try_call(self, train_resources);
                match call {
                    Ok(fut) => Some(Some(Ok(fut.await))),
                    Err(e) => Some(Some(Err(e))),
                }
            } else {
                Some(None)
            }
        } else {
            None
        }
    }

    /// Runs the clean function.
    pub async fn clean_visit<'f>(
        &'f mut self,
        train_resources: &'f TrainResources<E>,
    ) -> Option<Result<Result<(), E>, BorrowFail>> {
        let work_fn = self
            .spec
            .station_op
            .clean_fns()
            .map(|clean_fns| clean_fns.work_fn.clone());
        if let Some(work_fn) = work_fn {
            let call = work_fn.f.try_call(self, train_resources);
            let result = match call {
                Ok(fut) => Ok(fut.await),
                Err(e) => Err(e),
            };
            Some(result)
        } else {
            None
        }
    }
}

use std::{fmt, marker::PhantomData};

use choochoo_cfg_model::rt::{CheckStatus, StationMutRef, TrainResources};
use choochoo_rt_model::{error::StationSpecError, CleanEnsureOutcomeErr, CleanEnsureOutcomeOk};

/// Logic that conditionally executes an operation's clean functions.
#[derive(Debug)]
pub struct CleanDriver<E> {
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> CleanDriver<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    /// Processes a station clean operation.
    ///
    /// The algorithm is as follows:
    ///
    /// 1. Check whether the station is already cleaned.
    /// 2. If it is, return `Ok`.
    /// 3. If it isn't, run the clean function.
    /// 4. If it fails, return the error.
    /// 5. If it succeeds, check that the station is in the desired state.
    /// 6. If it isn't, store this as an error to return to the caller.
    /// 7. Return `Ok`.
    pub async fn ensure(
        station: &mut StationMutRef<'_, E>,
        train_resources: &TrainResources<E>,
    ) -> Result<CleanEnsureOutcomeOk, CleanEnsureOutcomeErr<E>>
    where
        E: From<StationSpecError>,
    {
        if let Some(check_fns) = station.clean_check(train_resources).await {
            let work_required = if let Some(check_status) = check_fns {
                check_status
                    .map_err(CleanEnsureOutcomeErr::CheckBorrowFail)?
                    .map_err(CleanEnsureOutcomeErr::CheckFail)?
                    == CheckStatus::WorkRequired
            } else {
                // if there is no check function, always do the work.
                true
            };

            if work_required {
                station
                    .clean_visit(train_resources)
                    .await
                    .ok_or(CleanEnsureOutcomeErr::Never)?
                    .map_err(CleanEnsureOutcomeErr::VisitBorrowFail)?
                    .map_err(|error| CleanEnsureOutcomeErr::WorkFail { error })?;

                // After we visit, if the check function reports we still
                // need to visit, then the visit function or the check
                // function needs to be corrected.
                let check_status =
                    if let Some(check_status) = station.clean_check(train_resources).await {
                        Some(
                            check_status
                                .ok_or(CleanEnsureOutcomeErr::Never)?
                                .map_err(CleanEnsureOutcomeErr::CheckBorrowFail)?
                                .map_err(CleanEnsureOutcomeErr::CheckFail)?,
                        )
                    } else {
                        None
                    };

                let station_spec_error = if let Some(CheckStatus::WorkRequired) = check_status {
                    let id = station.spec.id().clone();
                    let name = station.spec.name().to_string();
                    Some(StationSpecError::WorkRequiredAfterVisit { id, name })
                } else {
                    None
                };

                Ok(CleanEnsureOutcomeOk::Changed { station_spec_error })
            } else {
                Ok(CleanEnsureOutcomeOk::Unchanged)
            }
        } else {
            Ok(CleanEnsureOutcomeOk::NothingToDo)
        }
    }
}

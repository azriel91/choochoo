use std::{fmt, marker::PhantomData};

use choochoo_cfg_model::rt::{CheckStatus, ResourceIds, StationMutRef, TrainReport};
use choochoo_rt_model::{error::StationSpecError, EnsureOutcomeErr, EnsureOutcomeOk};

/// Logic that conditionally executes an operation's work.
#[derive(Debug)]
pub struct Driver<E> {
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> Driver<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    /// Processes a station operation.
    ///
    /// The algorithm is as follows:
    ///
    /// 1. Check whether the station is already in the desired state.
    /// 2. If it is, return `Ok`.
    /// 3. If it isn't, run the operation function.
    /// 4. If it fails, return the error.
    /// 5. If it succeeds, check that the station is in the desired state.
    /// 6. If it isn't, store this as an error to return to the caller.
    /// 7. Return `Ok`.
    ///
    /// # Implementation Note
    ///
    /// Other things to consider are:
    ///
    /// * Recording the timestamps / duration of each step.
    /// * Forwarding output to the user.
    /// * Serializing state to disk.
    pub async fn ensure(
        station: &mut StationMutRef<'_, E>,
        train_report: &TrainReport<E>,
    ) -> (
        Option<ResourceIds>,
        Result<EnsureOutcomeOk, EnsureOutcomeErr<E>>,
    )
    where
        E: From<StationSpecError>,
    {
        match Self::work_required(station, train_report).await {
            Ok(true) => {
                match station.visit(train_report).await {
                    Ok((resource_ids, visit_result)) => {
                        let visit_result = visit_result.map_err(EnsureOutcomeErr::WorkFail);

                        match visit_result {
                            Ok(()) => {
                                // After we visit, if the check function reports we still
                                // need to visit, then the visit function or the check
                                // function needs to be corrected.
                                let work_required =
                                    Self::work_required(station, train_report).await;

                                let station_spec_error = if let Some(check_status) =
                                    Self::check_status(station, train_report).await
                                {
                                    match check_status {
                                        Ok(CheckStatus::WorkRequired) => {
                                            let id = station.spec.id().clone();
                                            let name = station.spec.name().to_string();
                                            Some(StationSpecError::WorkRequiredAfterVisit {
                                                id,
                                                name,
                                            })
                                        }
                                        Ok(CheckStatus::WorkNotRequired) | Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                (
                                    Some(resource_ids),
                                    Ok(EnsureOutcomeOk::Changed { station_spec_error }),
                                )
                            }
                            Err(e) => (Some(resource_ids), Err(e)),
                        }
                    }
                    Err(e) => (None, Err(EnsureOutcomeErr::VisitBorrowFail(e))),
                }
            }
            Ok(false) => (None, Ok(EnsureOutcomeOk::Unchanged)),
            Err(e) => (None, Err(e)),
        }
    }

    async fn work_required(
        station: &mut StationMutRef<'_, E>,
        train_report: &TrainReport<E>,
    ) -> Result<bool, EnsureOutcomeErr<E>> {
        if let Some(check_status) = Self::check_status(station, train_report).await {
            let work_required = check_status? == CheckStatus::WorkRequired;
            Ok(work_required)
        } else {
            // if there is no check function, always do the work.
            Ok(true)
        }
    }

    async fn check_status(
        station: &mut StationMutRef<'_, E>,
        train_report: &TrainReport<E>,
    ) -> Option<Result<CheckStatus, EnsureOutcomeErr<E>>> {
        if let Some(check_status) = station.check(train_report).await {
            let check_status = check_status
                .map_err(EnsureOutcomeErr::CheckBorrowFail)
                .and_then(|check_status_result| {
                    check_status_result.map_err(EnsureOutcomeErr::CheckFail)
                });

            Some(check_status)
        } else {
            None
        }
    }
}

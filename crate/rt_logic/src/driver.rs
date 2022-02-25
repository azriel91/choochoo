use std::{fmt, marker::PhantomData};

use choochoo_cfg_model::rt::{CheckStatus, StationMutRef, TrainReport};
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
    ) -> Result<EnsureOutcomeOk, EnsureOutcomeErr<E>>
    where
        E: From<StationSpecError>,
    {
        let work_required = if let Some(check_status) = station.check(train_report).await {
            check_status
                .map_err(EnsureOutcomeErr::CheckBorrowFail)?
                .map_err(EnsureOutcomeErr::CheckFail)?
                == CheckStatus::WorkRequired
        } else {
            // if there is no check function, always do the work.
            true
        };

        if work_required {
            let resource_ids = station
                .visit(train_report)
                .await
                .map_err(EnsureOutcomeErr::VisitBorrowFail)?
                .map_err(|(resource_ids, error)| EnsureOutcomeErr::WorkFail {
                    resource_ids,
                    error,
                })?;

            // After we visit, if the check function reports we still
            // need to visit, then the visit function or the check
            // function needs to be corrected.
            let check_status = if let Some(check_status) = station.check(train_report).await {
                Some(
                    check_status
                        .map_err(EnsureOutcomeErr::CheckBorrowFail)?
                        .map_err(EnsureOutcomeErr::CheckFail)?,
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

            Ok(EnsureOutcomeOk::Changed {
                resource_ids,
                station_spec_error,
            })
        } else {
            Ok(EnsureOutcomeOk::Unchanged)
        }
    }
}

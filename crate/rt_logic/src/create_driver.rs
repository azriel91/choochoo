use std::{fmt, marker::PhantomData};

use choochoo_cfg_model::rt::{CheckStatus, StationMutRef, TrainResources};
use choochoo_rt_model::{error::StationSpecError, CreateEnsureOutcomeErr, CreateEnsureOutcomeOk};

/// Logic that conditionally executes an operation's work.
#[derive(Debug)]
pub struct CreateDriver<E> {
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> CreateDriver<E>
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
        train_resources: &TrainResources<E>,
    ) -> Result<CreateEnsureOutcomeOk, CreateEnsureOutcomeErr<E>>
    where
        E: From<StationSpecError>,
    {
        let work_required = if let Some(check_status) = station.create_check(train_resources).await
        {
            check_status
                .map_err(CreateEnsureOutcomeErr::CheckBorrowFail)?
                .map_err(CreateEnsureOutcomeErr::CheckFail)?
                == CheckStatus::WorkRequired
        } else {
            // if there is no check function, always do the work.
            true
        };

        if work_required {
            let res_ids = station
                .create_visit(train_resources)
                .await
                .map_err(CreateEnsureOutcomeErr::VisitBorrowFail)?
                .map_err(|(res_ids, error)| CreateEnsureOutcomeErr::WorkFail { res_ids, error })?;

            // After we visit, if the check function reports we still
            // need to visit, then the visit function or the check
            // function needs to be corrected.
            let check_status =
                if let Some(check_status) = station.create_check(train_resources).await {
                    Some(
                        check_status
                            .map_err(CreateEnsureOutcomeErr::CheckBorrowFail)?
                            .map_err(CreateEnsureOutcomeErr::CheckFail)?,
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

            Ok(CreateEnsureOutcomeOk::Changed {
                res_ids,
                station_spec_error,
            })
        } else {
            Ok(CreateEnsureOutcomeOk::Unchanged)
        }
    }
}

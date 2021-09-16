use std::{fmt, marker::PhantomData};

use choochoo_cfg_model::CheckStatus;
use choochoo_rt_model::{
    error::StationSpecError, EnsureOutcomeErr, EnsureOutcomeOk, StationMut, TrainReport,
};

/// Logic that determines whether or not to visit a station.
#[derive(Debug)]
pub struct Driver<E> {
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> Driver<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    /// Processes a station visit.
    ///
    /// The algorithm is as follows:
    ///
    /// 1. Check whether the station is already in the desired state.
    /// 2. If it is, return `Ok`.
    /// 3. If it isn't, run the visit function.
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
        station: &mut StationMut<'_, E>,
        train_report: &TrainReport<E>,
    ) -> Result<EnsureOutcomeOk, EnsureOutcomeErr<E>>
    where
        E: From<StationSpecError>,
    {
        let visit_required = if let Some(check_status) =
            station.spec.check(&mut station.progress, train_report)
        {
            check_status.await.map_err(EnsureOutcomeErr::CheckFail)? == CheckStatus::VisitRequired
        } else {
            // if there is no check function, always visit the station.
            true
        };

        if visit_required {
            station
                .spec
                .visit(&mut station.progress, train_report)
                .await
                .map_err(EnsureOutcomeErr::VisitFail)?;

            // After we visit, if the check function reports we still
            // need to visit, then the visit function or the check
            // function needs to be corrected.
            let station_spec_error = if let Some(check_status) =
                station.spec.check(&mut station.progress, train_report)
            {
                let check_status = check_status.await.map_err(EnsureOutcomeErr::CheckFail)?;
                if check_status == CheckStatus::VisitRequired {
                    let id = station.spec.id().clone();
                    let name = station.spec.name().to_string();
                    Some(StationSpecError::VisitRequiredAfterVisit { id, name })
                } else {
                    None
                }
            } else {
                None
            };

            Ok(EnsureOutcomeOk::Changed { station_spec_error })
        } else {
            Ok(EnsureOutcomeOk::Unchanged)
        }
    }
}

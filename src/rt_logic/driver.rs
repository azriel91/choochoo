use std::marker::PhantomData;

use resman::Resources;

use crate::{
    cfg_model::{CheckStatus, StationProgress, StationSpec},
    rt_model::{error::StationSpecError, EnsureOutcomeErr, EnsureOutcomeOk},
};

/// Logic that determines whether or not to visit a station.
#[derive(Debug)]
pub struct Driver<E> {
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> Driver<E> {
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
        station_spec: &StationSpec<E>,
        station_progress: &mut StationProgress<E>,
        resources: &Resources,
    ) -> Result<EnsureOutcomeOk, EnsureOutcomeErr<E>>
    where
        E: From<StationSpecError>,
    {
        let visit_required = if let Some(check_status) =
            station_spec.check(station_progress, resources)
        {
            check_status.await.map_err(EnsureOutcomeErr::CheckFail)? == CheckStatus::VisitRequired
        } else {
            // if there is no check function, always visit the station.
            true
        };

        if visit_required {
            station_spec
                .visit(station_progress, resources)
                .await
                .map_err(EnsureOutcomeErr::VisitFail)?;

            // After we visit, if the check function reports we still
            // need to visit, then the visit function or the check
            // function needs to be corrected.
            let spec_has_error =
                if let Some(check_status) = station_spec.check(station_progress, resources) {
                    check_status.await.map_err(EnsureOutcomeErr::CheckFail)?
                        == CheckStatus::VisitRequired
                } else {
                    false
                };

            // Need to split this out, because `station_progress` is borrowed during the
            // scope of the `if let`
            if spec_has_error {
                let id = station_spec.id().clone();
                let name = station_spec.name().to_string();
                let station_spec_error = StationSpecError::VisitRequiredAfterVisit { id, name };

                station_progress.error = Some(E::from(station_spec_error));
            }

            Ok(EnsureOutcomeOk::Changed)
        } else {
            Ok(EnsureOutcomeOk::Unchanged)
        }
    }
}

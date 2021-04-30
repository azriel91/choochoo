use std::marker::PhantomData;

use daggy::{petgraph::graph::DefaultIx, NodeIndex};

use crate::{
    cfg_model::CheckStatus,
    rt_model::{error::StationSpecError, EnsureOutcome, Station, TrainReport},
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
        train_report: &mut TrainReport<E>,
        node_id: NodeIndex<DefaultIx>,
        station: &mut Station<E>,
    ) -> Result<EnsureOutcome, E>
    where
        E: From<StationSpecError>,
    {
        let TrainReport { errors, resources } = train_report;

        let visit_required = if let Some(check_status) = station.check(resources) {
            check_status.await? == CheckStatus::VisitRequired
        } else {
            // if there is no check function, always visit the station.
            true
        };

        if visit_required {
            station.visit(&resources).await?;

            // After we visit, if the check function reports we still
            // need to visit, then the visit function or the check
            // function needs to be corrected.
            let spec_has_error = if let Some(check_status) = station.check(resources) {
                check_status.await? == CheckStatus::VisitRequired
            } else {
                false
            };

            // Need to split this out, because `station` is borrowed during the scope of the
            // `if let`
            if spec_has_error {
                let id = station.station_spec.id().clone();
                let name = station.station_spec.name().to_string();
                let station_spec_error = StationSpecError::VisitRequiredAfterVisit { id, name };

                errors.insert(node_id, E::from(station_spec_error));
            }

            Ok(EnsureOutcome::Changed)
        } else {
            Ok(EnsureOutcome::Unchanged)
        }
    }
}

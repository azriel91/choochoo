use std::collections::HashMap;

use crate::{
    cfg_model::StationId,
    rt_model::{StationProgresses, StationRtId, Stations},
};

/// Specification of a desired state.
#[derive(Debug, Default)]
pub struct Destination<E> {
    /// The stations along the way to the destination.
    stations: Stations<E>,
    /// Map from station ID to station runtime ID.
    ///
    /// This is the only clone of `StationId`s that we should hold.
    station_id_to_rt_id: HashMap<StationId, StationRtId>,
    /// Progress information for each `Station`.
    station_progresses: StationProgresses<E>,
}

impl<E> Destination<E> {
    /// Returns a new `Destination`.
    ///
    /// # Parameters
    ///
    /// * `stations`: The stations along the way to the destination.
    /// * `station_progresses`: The initial state of the stations.
    pub fn new(stations: Stations<E>, station_progresses: StationProgresses<E>) -> Self {
        let mut station_id_to_rt_id = HashMap::with_capacity(stations.node_count());
        stations
            .iter_with_indices()
            .for_each(|(node_index, station_spec)| {
                station_id_to_rt_id.insert(station_spec.id().clone(), node_index);
            });

        Self {
            stations,
            station_id_to_rt_id,
            station_progresses,
        }
    }

    /// Returns a reference to the `Stations` for this destination.
    pub fn stations(&self) -> &Stations<E> {
        &self.stations
    }

    /// Returns a mutable reference to the `Stations` for this destination.
    pub fn stations_mut(&mut self) -> &mut Stations<E> {
        &mut self.stations
    }

    /// Returns a reference to the station progresses.
    pub fn station_progresses(&self) -> &StationProgresses<E> {
        &self.station_progresses
    }

    /// Returns a mutable reference to the station progresses.
    pub fn station_progresses_mut(&mut self) -> &mut StationProgresses<E> {
        &mut self.station_progresses
    }

    /// Returns a reference to the station ID to runtime ID map.
    pub fn station_id_to_rt_id(&self) -> &HashMap<StationId, StationRtId> {
        &self.station_id_to_rt_id
    }
}

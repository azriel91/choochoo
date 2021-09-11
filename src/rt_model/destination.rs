use std::collections::HashMap;

use crate::{
    cfg_model::StationId,
    rt_model::{Station, StationMut, StationProgresses, StationRtId, StationSpecs},
};

/// Specification of a desired state.
#[derive(Debug, Default)]
pub struct Destination<E> {
    /// The stations along the way to the destination.
    station_specs: StationSpecs<E>,
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
    /// * `station_specs`: The stations along the way to the destination.
    /// * `station_progresses`: The initial state of the stations.
    pub fn new(station_specs: StationSpecs<E>, station_progresses: StationProgresses<E>) -> Self {
        let mut station_id_to_rt_id = HashMap::with_capacity(station_specs.node_count());
        station_specs
            .iter_with_indices()
            .for_each(|(node_index, station_spec)| {
                station_id_to_rt_id.insert(station_spec.id().clone(), node_index);
            });

        Self {
            station_specs,
            station_id_to_rt_id,
            station_progresses,
        }
    }

    /// Returns an iterator over the [`StationMut`]s in this destination.
    ///
    /// This uses runtime borrowing ([`RtMap::try_borrow`]) to retrieve the
    /// station progress, so if a station's progress is already accessed, then
    /// it will not be returned by the iterator.
    ///
    /// [`RtMap::try_borrow`]: rt_map::RtMap::try_borrow
    pub fn stations(&self) -> impl Iterator<Item = Station<'_, E>> + '_ {
        self.station_specs.iter().filter_map(move |station_spec| {
            self.station_id_to_rt_id
                .get(station_spec.id())
                .and_then(|station_rt_id| {
                    self.station_progresses
                        .try_borrow(station_rt_id)
                        .map(|station_progress| (*station_rt_id, station_progress))
                })
                .map(|(station_rt_id, station_progress)| Station {
                    spec: station_spec,
                    rt_id: station_rt_id,
                    progress: station_progress,
                })
        })
    }

    /// Returns an iterator over the [`StationMut`]s in this destination.
    ///
    /// This uses runtime borrowing ([`RtMap::try_borrow_mut`]) to retrieve the
    /// station progress, so if a station's progress is already accessed, then
    /// it will not be returned by the iterator.
    ///
    /// [`RtMap::try_borrow_mut`]: rt_map::RtMap::try_borrow_mut
    pub fn stations_mut(&self) -> impl Iterator<Item = StationMut<'_, E>> + '_ {
        self.station_specs.iter().filter_map(move |station_spec| {
            self.station_id_to_rt_id
                .get(station_spec.id())
                .and_then(|station_rt_id| {
                    self.station_progresses
                        .try_borrow_mut(station_rt_id)
                        .map(|station_progress| (*station_rt_id, station_progress))
                })
                .map(|(station_rt_id, station_progress)| StationMut {
                    spec: station_spec,
                    rt_id: station_rt_id,
                    progress: station_progress,
                })
        })
    }

    /// Returns a reference to the [`StationSpecs`] for this destination.
    pub fn station_specs(&self) -> &StationSpecs<E> {
        &self.station_specs
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

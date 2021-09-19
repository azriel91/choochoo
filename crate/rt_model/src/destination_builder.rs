use std::collections::HashMap;

use choochoo_cfg_model::{
    daggy::{EdgeIndex, WouldCycle},
    indicatif::ProgressStyle,
    StationProgress, StationSpec, StationSpecs, VisitStatus, Workload,
};

use crate::{Destination, StationProgresses, StationRtId};

#[derive(Debug)]
pub struct DestinationBuilder<E> {
    /// The stations along the way to the destination.
    station_specs: StationSpecs<E>,
    /// Progress information for each `Station`.
    station_progresses: StationProgresses,
}

impl<E> DestinationBuilder<E> {
    /// Returns a new `DestinationBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a station to this destination.
    ///
    /// The returned station ID is used to specify dependencies between stations
    /// through the [`add_edge`] method.
    ///
    /// [`add_edge`]: Self::add_edge
    pub fn add_station(&mut self, station_spec: StationSpec<E>) -> StationRtId {
        let station_progress = StationProgress::new(&station_spec, VisitStatus::Queued)
            // TODO: User specifies which kind of style they want, and we set the `ProgressStyle`
            // accordingly.
            .with_progress_style(
                ProgressStyle::default_bar()
                    .template(StationProgress::STYLE_IN_PROGRESS_BYTES)
                    .progress_chars("█▉▊▋▌▍▎▏  "),
            );
        let station_rt_id = self.station_specs.add_node(station_spec);
        self.station_progresses
            .insert(station_rt_id, station_progress);

        station_rt_id
    }

    /// Adds an edge from one station to another.
    ///
    /// This differs from [`petgraph`'s `add_edge`] in that this only allows one
    /// edge between two stations. When this function is called multiple times
    /// with the same stations, only the last call's edge will be retained.
    ///
    /// [`petgraph`'s `add_edge`]:
    /// choochoo_cfg_model::daggy::petgraph::data::Build::add_edge
    pub fn add_edge(
        &mut self,
        station_from: StationRtId,
        station_to: StationRtId,
        edge: Workload,
    ) -> Result<EdgeIndex, WouldCycle<Workload>> {
        // Use `update_edge` instead of `add_edge` to avoid duplicate edges from one
        // station to the other.
        self.station_specs
            .update_edge(station_from, station_to, edge)
    }

    /// Adds edges between stations.
    pub fn extend_with_edges<I>(&mut self, edges: I) -> Result<(), WouldCycle<Workload>>
    where
        I: IntoIterator<Item = (StationRtId, StationRtId, Workload)>,
    {
        edges
            .into_iter()
            .try_for_each(|(station_from, station_to, edge)| {
                self.add_edge(station_from, station_to, edge)
                    .map(|_edge_index| ())
            })
    }

    /// Builds and returns the [`Destination`].
    pub fn build(self) -> Destination<E> {
        let Self {
            station_specs,
            station_progresses,
        } = self;

        let mut station_id_to_rt_id = HashMap::with_capacity(station_specs.node_count());
        station_specs
            .iter_with_indices()
            .for_each(|(node_index, station_spec)| {
                station_id_to_rt_id.insert(station_spec.id().clone(), node_index);
            });

        Destination {
            station_specs,
            station_id_to_rt_id,
            station_progresses,
        }
    }
}

impl<E> Default for DestinationBuilder<E> {
    fn default() -> Self {
        Self {
            station_specs: StationSpecs::default(),
            station_progresses: StationProgresses::default(),
        }
    }
}

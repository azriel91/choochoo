use std::collections::HashMap;

use choochoo_cfg_model::{
    daggy::WouldCycle,
    fn_graph::{Edge, EdgeId, FnGraphBuilder},
    rt::{ProgressLimit, StationProgress, StationRtId},
    StationSpec, StationSpecs,
};
use choochoo_resource::Profile;

use crate::{Destination, DestinationDirCalc, Error, StationProgresses, WorkspaceSpec};

#[derive(Debug)]
pub struct DestinationBuilder<E> {
    /// Execution profile identifier.
    profile: Option<Profile>,
    /// Describes how to discover the workspace directory.
    ///
    /// By default the execution working directory is used.
    workspace_spec: Option<WorkspaceSpec>,
    /// Builder for the stations along the way to the destination.
    fn_graph_builder: FnGraphBuilder<StationSpec<E>>,
}

impl<E> DestinationBuilder<E>
where
    E: 'static,
{
    /// Returns a new `DestinationBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies the execution profile identifier.
    #[must_use]
    pub fn with_profile(mut self, profile: Profile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Specifies how to discover the workspace directory.
    ///
    /// By default the execution working directory is used.
    #[must_use]
    pub fn with_workspace_spec(mut self, workspace_spec: WorkspaceSpec) -> Self {
        self.workspace_spec = Some(workspace_spec);
        self
    }

    /// Adds a station to this destination.
    ///
    /// The returned station ID is used to specify dependencies between stations
    /// through the [`add_edge`] method.
    ///
    /// [`add_edge`]: Self::add_edge
    pub fn add_station(&mut self, station_spec: StationSpec<E>) -> StationRtId {
        self.fn_graph_builder.add_fn(station_spec)
    }

    /// Adds multiple stations to this destination.
    ///
    /// The returned station IDs are used to specify dependencies between
    /// stations through the [`add_edge`] / [`add_edges`] method.
    ///
    /// [`add_edge`]: Self::add_edge
    /// [`add_edges`]: Self::add_edges
    pub fn add_stations<const N: usize>(
        &mut self,
        station_specs: [StationSpec<E>; N],
    ) -> [StationRtId; N] {
        self.fn_graph_builder.add_fns(station_specs)
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
    ) -> Result<EdgeId, WouldCycle<Edge>> {
        self.fn_graph_builder.add_edge(station_from, station_to)
    }

    /// Adds edges between stations.
    pub fn add_edges<const N: usize>(
        &mut self,
        edges: [(StationRtId, StationRtId); N],
    ) -> Result<[EdgeId; N], WouldCycle<Edge>> {
        self.fn_graph_builder.add_edges(edges)
    }

    /// Builds and returns the [`Destination`].
    pub fn build(self) -> Result<Destination<E>, Error<E>> {
        let Self {
            profile,
            workspace_spec,
            fn_graph_builder,
        } = self;

        let profile = profile.unwrap_or_default();
        let workspace_spec = workspace_spec.unwrap_or_default();
        let station_specs = StationSpecs::new(fn_graph_builder.build());

        let destination_dirs = DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

        let mut station_id_to_rt_id = HashMap::with_capacity(station_specs.node_count());
        station_specs
            .iter_insertion_with_indices()
            .for_each(|(node_index, station_spec)| {
                station_id_to_rt_id.insert(station_spec.id().clone(), node_index);
            });

        let station_progresses = station_specs
            .iter_insertion_with_indices()
            .map(|(station_rt_id, station_spec)| {
                let station_progress = StationProgress::new(station_spec, ProgressLimit::Unknown);
                (station_rt_id, station_progress)
            })
            .fold(
                StationProgresses::with_capacity(station_specs.node_count()),
                |mut station_progresses, (station_rt_id, station_progress)| {
                    station_progresses.insert(station_rt_id, station_progress);
                    station_progresses
                },
            );

        let dest = Destination {
            profile,
            station_specs,
            dirs: destination_dirs,
            station_id_to_rt_id,
            station_progresses,
        };
        Ok(dest)
    }
}

impl<E> Default for DestinationBuilder<E> {
    fn default() -> Self {
        Self {
            profile: None,
            workspace_spec: None,
            fn_graph_builder: FnGraphBuilder::default(),
        }
    }
}

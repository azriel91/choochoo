use std::{collections::HashMap, mem::MaybeUninit};

use choochoo_cfg_model::{
    daggy::{EdgeIndex, WouldCycle},
    ProgressLimit, StationProgress, StationRtId, StationSpec, StationSpecs, Workload,
};

use crate::{Destination, StationProgresses};

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
        let station_progress = StationProgress::new(&station_spec, ProgressLimit::Unknown);
        let station_rt_id = self.station_specs.add_node(station_spec);
        self.station_progresses
            .insert(station_rt_id, station_progress);

        station_rt_id
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
        // Create an uninitialized array of `MaybeUninit`. The `assume_init` is safe
        // because the type we are claiming to have initialized here is a bunch of
        // `MaybeUninit`s, which do not require initialization.
        //
        // https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        //
        // Switch this to `MaybeUninit::uninit_array` once it is stable.
        let mut station_rt_ids: [MaybeUninit<StationRtId>; N] =
            unsafe { MaybeUninit::uninit().assume_init() };

        IntoIterator::into_iter(station_specs)
            .map(|station_spec| self.add_station(station_spec))
            .zip(station_rt_ids.iter_mut())
            .for_each(|(station_rt_id, station_rt_id_mem)| {
                station_rt_id_mem.write(station_rt_id);
            });

        // Everything is initialized. Transmute the array to the initialized type.
        // Unfortunately we cannot use this, see the following issues:
        //
        // * <https://github.com/rust-lang/rust/issues/61956>
        // * <https://github.com/rust-lang/rust/issues/80908>
        //
        // let station_rt_ids = unsafe { mem::transmute::<_, [StationRtId;
        // N]>(station_rt_ids) };

        #[allow(clippy::let_and_return)] // for clarity with `unsafe`
        let station_rt_ids = {
            let ptr = &mut station_rt_ids as *mut _ as *mut [StationRtId; N];
            let array = unsafe { ptr.read() };

            // We don't have to `mem::forget` the original because `StationRtId` is `Copy`.
            // mem::forget(station_rt_ids);

            array
        };

        station_rt_ids
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
    pub fn add_edges<const N: usize>(
        &mut self,
        edges: [(StationRtId, StationRtId, Workload); N],
    ) -> Result<[EdgeIndex; N], WouldCycle<Workload>> {
        // Create an uninitialized array of `MaybeUninit`. The `assume_init` is safe
        // because the type we are claiming to have initialized here is a bunch of
        // `MaybeUninit`s, which do not require initialization.
        //
        // https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        //
        // Switch this to `MaybeUninit::uninit_array` once it is stable.
        let mut edge_indicies: [MaybeUninit<EdgeIndex>; N] =
            unsafe { MaybeUninit::uninit().assume_init() };

        IntoIterator::into_iter(edges)
            .zip(edge_indicies.iter_mut())
            .try_for_each(|((station_from, station_to, edge), edge_index_mem)| {
                self.add_edge(station_from, station_to, edge)
                    .map(|edge_index| {
                        edge_index_mem.write(edge_index);
                    })
            })?;

        // Everything is initialized. Transmute the array to the initialized type.
        // Unfortunately we cannot use this, see the following issues:
        //
        // * <https://github.com/rust-lang/rust/issues/61956>
        // * <https://github.com/rust-lang/rust/issues/80908>
        //
        // let edge_indicies = unsafe { mem::transmute::<_, [EdgeIndex;
        // N]>(edge_indicies) };

        #[allow(clippy::let_and_return)] // for clarity with `unsafe`
        let edge_indicies = {
            let ptr = &mut edge_indicies as *mut _ as *mut [EdgeIndex; N];
            let array = unsafe { ptr.read() };

            // We don't have to `mem::forget` the original because `EdgeIndex` is `Copy`.
            // mem::forget(edge_indicies);

            array
        };

        Ok(edge_indicies)
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

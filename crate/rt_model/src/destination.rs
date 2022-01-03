use std::collections::HashMap;

use choochoo_cfg_model::{
    rt::{Station, StationMut, StationMutRef, StationRtId},
    StationId, StationSpecs,
};
use choochoo_resource::{Profile, ProfileDir, WorkspaceDir};
use futures::{stream::Stream, StreamExt};

use crate::{DestinationBuilder, StationDirs, StationProgresses};

/// Specification of a desired state.
#[derive(Debug)]
pub struct Destination<E> {
    /// Base directory of the workspace.
    pub(crate) workspace_dir: WorkspaceDir,
    /// Execution profile identifier.
    pub(crate) profile: Profile,
    /// Directory to store all data produced by an execution.
    pub(crate) profile_dir: ProfileDir,
    /// Map from [`StationRtId`] to the station's execution directory.
    pub(crate) station_dirs: StationDirs,
    /// The stations along the way to the destination.
    pub(crate) station_specs: StationSpecs<E>,
    /// Map from station ID to station runtime ID.
    ///
    /// This is the only clone of `StationId`s that we should hold.
    pub(crate) station_id_to_rt_id: HashMap<StationId, StationRtId>,
    /// Progress information for each `Station`.
    pub(crate) station_progresses: StationProgresses,
}

impl<E> Destination<E>
where
    E: 'static,
{
    /// Returns a new `DestinationBuilder`.
    pub fn builder() -> DestinationBuilder<E> {
        DestinationBuilder::new()
    }

    /// Returns the base directory of the workspace.
    pub fn workspace_dir(&self) -> &WorkspaceDir {
        &self.workspace_dir
    }

    /// Returns the profile.
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Returns the directory to store all data produced by an execution.
    pub fn profile_dir(&self) -> &ProfileDir {
        &self.profile_dir
    }

    /// Returns a reference to the station directories.
    pub fn station_dirs(&self) -> &StationDirs {
        &self.station_dirs
    }

    /// Returns an iterator over the [`Station`]s in this destination.
    ///
    /// This uses runtime borrowing ([`RtMap::try_borrow`]) to retrieve the
    /// station progress, so if a station's progress is already accessed
    /// mutably, then it will not be returned by the iterator.
    ///
    /// [`RtMap::try_borrow`]: rt_map::RtMap::try_borrow
    pub fn stations(&self) -> impl Iterator<Item = Station<'_, E>> + '_ {
        self.station_specs
            .iter_insertion()
            .filter_map(move |station_spec| {
                self.station_id_to_rt_id
                    .get(station_spec.id())
                    .and_then(|station_rt_id| {
                        self.station_progresses
                            .try_borrow(station_rt_id)
                            .map(|station_progress| (*station_rt_id, station_progress))
                            .ok()
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
        self.station_specs
            .iter_insertion()
            .filter_map(move |station_spec| {
                self.station_id_to_rt_id
                    .get(station_spec.id())
                    .and_then(|station_rt_id| {
                        let station_dir = self.station_dirs.get(station_rt_id);
                        let station_progress =
                            self.station_progresses.try_borrow_mut(station_rt_id);

                        if let (Some(station_dir), Ok(station_progress)) =
                            (station_dir, station_progress)
                        {
                            Some((*station_rt_id, station_dir, station_progress))
                        } else {
                            None
                        }
                    })
                    .map(
                        |(station_rt_id, station_dir, station_progress)| StationMut {
                            spec: station_spec,
                            rt_id: station_rt_id,
                            dir: station_dir,
                            progress: station_progress,
                        },
                    )
            })
    }

    /// Returns an iterator over the [`Station`]s in this destination in
    /// dependency order.
    ///
    /// This uses runtime borrowing ([`RtMap::try_borrow`]) to retrieve the
    /// station progress, so if a station's progress is already accessed
    /// mutably, then it will not be returned by the iterator.
    ///
    /// [`RtMap::try_borrow`]: rt_map::RtMap::try_borrow
    pub fn stations_iter(&self) -> impl Iterator<Item = Station<'_, E>> + '_ {
        self.station_specs.iter().filter_map(move |station_spec| {
            self.station_id_to_rt_id
                .get(station_spec.id())
                .and_then(|station_rt_id| {
                    self.station_progresses
                        .try_borrow(station_rt_id)
                        .map(|station_progress| (*station_rt_id, station_progress))
                        .ok()
                })
                .map(|(station_rt_id, station_progress)| Station {
                    spec: station_spec,
                    rt_id: station_rt_id,
                    progress: station_progress,
                })
        })
    }

    /// Returns an iterator over the [`StationMutRef`]s in this destination.
    ///
    /// This uses runtime borrowing ([`RtMap::try_borrow_mut`]) to retrieve the
    /// station progress, so if a station's progress is already accessed, then
    /// it will not be returned by the iterator.
    ///
    /// [`RtMap::try_borrow_mut`]: rt_map::RtMap::try_borrow_mut
    pub fn stations_mut_stream(&self) -> impl Stream<Item = StationMutRef<'_, E>> + '_ {
        self.station_specs
            .stream()
            .filter_map(move |station_spec| async move {
                self.station_id_to_rt_id
                    .get(station_spec.id())
                    .and_then(|station_rt_id| {
                        let station_dir = self.station_dirs.get(station_rt_id);
                        let station_progress =
                            self.station_progresses.try_borrow_mut(station_rt_id);

                        if let (Some(station_dir), Ok(station_progress)) =
                            (station_dir, station_progress)
                        {
                            Some((*station_rt_id, station_dir, station_progress))
                        } else {
                            None
                        }
                    })
                    .map(
                        |(station_rt_id, station_dir, station_progress)| StationMutRef {
                            spec: station_spec,
                            rt_id: station_rt_id,
                            dir: station_dir,
                            progress: station_progress,
                        },
                    )
            })
    }

    /// Returns a reference to the [`StationSpecs`] for this destination.
    pub fn station_specs(&self) -> &StationSpecs<E> {
        &self.station_specs
    }

    /// Returns a reference to the station progresses.
    pub fn station_progresses(&self) -> &StationProgresses {
        &self.station_progresses
    }

    /// Returns a mutable reference to the station progresses.
    pub fn station_progresses_mut(&mut self) -> &mut StationProgresses {
        &mut self.station_progresses
    }

    /// Returns a reference to the station ID to runtime ID map.
    pub fn station_id_to_rt_id(&self) -> &HashMap<StationId, StationRtId> {
        &self.station_id_to_rt_id
    }
}

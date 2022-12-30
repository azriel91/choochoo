use std::{
    collections::HashMap,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use choochoo_cfg_model::{rt::StationDir, StationSpecs};
use choochoo_resource::{HistoryDir, Profile, ProfileDir, ProfileHistoryDir, WorkspaceDir};

use crate::{DestinationDirs, Error, StationDirs, WorkspaceSpec};

/// Computes directories for a destination.
#[derive(Debug)]
pub struct DestinationDirCalc<E>(PhantomData<E>);

impl<E> DestinationDirCalc<E>
where
    E: 'static,
{
    /// Directory to contain all profile history directories.
    pub const HISTORY_DIR_NAME: &'static str = ".history";
    /// Directory to contain all profile directories.
    pub const TARGET_DIR_NAME: &'static str = "target";

    /// Computes directories for a destination.
    ///
    /// This includes:
    ///
    /// * [`WorkspaceDir`]: `${workspace}`
    /// * [`HistoryDir`]: `${workspace}/target/.history`
    /// * [`ProfileHistoryDir`]: `${workspace}/target/.history/${profile}`
    /// * [`ProfileDir`]: `${workspace}/target/${profile}`
    /// * [`StationDirs`]: `${workspace}/target/${profile}/${station_id}`
    pub fn calc(
        workspace_spec: &WorkspaceSpec,
        profile: &Profile,
        station_specs: &StationSpecs<E>,
    ) -> Result<DestinationDirs, Error<E>> {
        let workspace_dir = {
            let working_dir = std::env::current_dir().map_err(Error::WorkingDirRead)?;
            let workspace_dir = match workspace_spec {
                WorkspaceSpec::WorkingDir => working_dir,
                WorkspaceSpec::FirstDirWithFile(file_name) => {
                    Self::first_dir_with_file(&working_dir, file_name).ok_or_else(move || {
                        let file_name = file_name.to_path_buf();
                        Error::WorkspaceFileNotFound {
                            working_dir,
                            file_name,
                        }
                    })?
                }
                WorkspaceSpec::Path(path) => path.clone(),
            };

            WorkspaceDir::new(workspace_dir)
        };

        let history_dir = HistoryDir::new(
            workspace_dir
                .join(Self::TARGET_DIR_NAME)
                .join(Self::HISTORY_DIR_NAME),
        );
        let profile_history_dir = ProfileHistoryDir::new(history_dir.join(profile.as_ref()));

        let profile_dir = ProfileDir::new(
            workspace_dir
                .join(Self::TARGET_DIR_NAME)
                .join(profile.as_ref()),
        );
        let station_dirs = {
            let station_dirs = station_specs.iter_insertion_with_indices().fold(
                HashMap::with_capacity(station_specs.node_count()),
                |mut station_dirs, (station_rt_id, station_spec)| {
                    let station_dir = StationDir::new(profile_dir.join(station_spec.id().as_ref()));

                    station_dirs.insert(station_rt_id, station_dir);
                    station_dirs
                },
            );

            StationDirs(station_dirs)
        };

        Ok(DestinationDirs {
            workspace_dir,
            history_dir,
            profile_history_dir,
            profile_dir,
            station_dirs,
        })
    }

    fn first_dir_with_file(working_dir: &Path, path: &Path) -> Option<PathBuf> {
        let mut candidate_dir = working_dir.to_path_buf();
        loop {
            let candidate_marker = candidate_dir.join(path);
            if candidate_marker.exists() {
                return Some(candidate_dir);
            }

            // pop() returns false if there is no parent dir.
            if !candidate_dir.pop() {
                return None;
            }
        }
    }
}

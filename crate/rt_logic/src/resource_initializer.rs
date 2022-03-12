use std::marker::PhantomData;

use choochoo_cfg_model::rt::TrainResources;
use choochoo_rt_model::{Destination, DestinationDirCalc, DestinationDirs, Error};
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio::fs;

/// Initializes execution resources and adds them to the train resources.
///
/// This includes:
///
/// * [`WorkspaceDir`]
/// * [`HistoryDir`]
/// * [`ProfileHistoryDir`]
/// * [`ProfileHistoryStationDirs`]
/// * [`Profile`]
/// * [`ProfileDir`]
/// * [`StationDirs`]
///
/// All directories are ensured to exist.
#[derive(Debug)]
pub struct ResourceInitializer<E>(PhantomData<E>);

impl<E> ResourceInitializer<E>
where
    E: 'static,
{
    /// Initializes execution resources and adds them to the train resources.
    ///
    /// This includes:
    ///
    /// * [`Profile`]
    /// * [`ProfileDir`]
    /// * [`WorkspaceDir`]
    ///
    /// The [`ProfileDir`] and [`StationDir`]s are ensured to exist.
    pub async fn initialize(
        dest: &Destination<E>,
        train_resources: &mut TrainResources<E>,
    ) -> Result<(), Error<E>> {
        let DestinationDirs {
            workspace_dir,
            history_dir,
            profile_history_dir,
            profile_history_station_dirs,
            profile_dir,
            station_dirs,
        } = dest.dirs().clone();
        let profile = dest.profile().clone();
        let target_dir = workspace_dir.join(DestinationDirCalc::<E>::TARGET_DIR_NAME);

        macro_rules! ensure_dir_exists {
            ($dir:ident, $error_variant:ident) => {
                if !$dir.exists() {
                    fs::create_dir_all(&$dir)
                        .await
                        .map_err(|error| Error::$error_variant {
                            $dir: $dir.clone(),
                            error,
                        })?;
                }
            };
        }

        ensure_dir_exists!(workspace_dir, WorkspaceDirCreate);
        ensure_dir_exists!(target_dir, TargetDirCreate);
        ensure_dir_exists!(history_dir, HistoryDirCreate);
        ensure_dir_exists!(profile_history_dir, ProfileHistoryDirCreate);
        stream::iter(profile_history_station_dirs.iter())
            .map(Result::<_, Error<E>>::Ok)
            .try_for_each_concurrent(4, |(_, profile_history_station_dir)| async move {
                ensure_dir_exists!(profile_history_station_dir, ProfileHistoryStationDirCreate);
                Ok(())
            })
            .await?;

        ensure_dir_exists!(profile_dir, ProfileDirCreate);
        stream::iter(station_dirs.iter())
            .map(Result::<_, Error<E>>::Ok)
            .try_for_each_concurrent(4, |(_, station_dir)| async move {
                ensure_dir_exists!(station_dir, StationDirCreate);
                Ok(())
            })
            .await?;

        train_resources.insert(workspace_dir);
        train_resources.insert(history_dir);
        train_resources.insert(profile_history_dir);
        train_resources.insert(profile_history_station_dirs);
        train_resources.insert(profile);
        train_resources.insert(profile_dir);
        train_resources.insert(station_dirs);

        Ok(())
    }
}

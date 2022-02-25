use std::marker::PhantomData;

use choochoo_cfg_model::rt::TrainResources;
use choochoo_rt_model::{Destination, DestinationDirCalc, Error};
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio::fs;

/// Initializes execution resources and adds them to the train resources.
///
/// This includes:
///
/// * [`Profile`]
/// * [`ProfileDir`]
/// * [`WorkspaceDir`]
///
/// The [`ProfileDir`] and [`StationDir`]s are ensured to exist.
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
        let workspace_dir = dest.workspace_dir().clone();
        let target_dir = workspace_dir.join(DestinationDirCalc::<E>::TARGET_DIR);
        let profile = dest.profile().clone();
        let profile_dir = dest.profile_dir().clone();
        let station_dirs = dest.station_dirs().clone();

        if !workspace_dir.exists() {
            fs::create_dir_all(&workspace_dir).await.map_err(|error| {
                Error::WorkspaceDirCreate {
                    workspace_dir: workspace_dir.clone(),
                    error,
                }
            })?;
        }
        if !target_dir.exists() {
            fs::create_dir(&target_dir)
                .await
                .map_err(|error| Error::ProfileDirCreate {
                    profile_dir: profile_dir.clone(),
                    error,
                })?;
        }
        if !profile_dir.exists() {
            fs::create_dir(&profile_dir)
                .await
                .map_err(|error| Error::ProfileDirCreate {
                    profile_dir: profile_dir.clone(),
                    error,
                })?;
        }
        stream::iter(station_dirs.iter())
            .map(Result::<_, Error<E>>::Ok)
            .try_for_each_concurrent(4, |(_, station_dir)| async move {
                if !station_dir.exists() {
                    fs::create_dir(station_dir)
                        .await
                        .map_err(|error| Error::StationDirCreate {
                            station_dir: station_dir.clone(),
                            error,
                        })
                } else {
                    Ok(())
                }
            })
            .await?;

        train_resources.insert(workspace_dir);
        train_resources.insert(profile);
        train_resources.insert(profile_dir);
        train_resources.insert(station_dirs);

        Ok(())
    }
}

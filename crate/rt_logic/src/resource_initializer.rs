use std::marker::PhantomData;

use choochoo_cfg_model::rt::TrainReport;

use choochoo_rt_model::{Destination, Error};

/// Initializes execution resources and adds them to the train report.
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
    /// Initializes execution resources and adds them to the train report.
    ///
    /// This includes:
    ///
    /// * [`Profile`]
    /// * [`ProfileDir`]
    /// * [`WorkspaceDir`]
    ///
    /// The [`ProfileDir`] and [`StationDir`]s are ensured to exist.
    pub fn initialize(
        dest: &Destination<E>,
        train_report: &mut TrainReport<E>,
    ) -> Result<(), Error<E>> {
        let workspace_dir = dest.workspace_dir().clone();
        let profile = dest.profile().clone();
        let profile_dir = dest.profile_dir().clone();
        let station_dirs = dest.station_dirs().clone();

        train_report.insert(workspace_dir);
        train_report.insert(profile);
        train_report.insert(profile_dir);
        train_report.insert(station_dirs);

        Ok(())
    }
}

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use choochoo_cfg_model::rt::TrainReport;
use choochoo_resource::{ProfileDir, WorkspaceDir};
use choochoo_rt_model::{Destination, Error, WorkspaceSpec};

/// Directory to contain all profile directories.
const TARGET_DIR: &str = "target";

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
        let profile = dest.profile().clone();

        let workspace_dir = {
            let working_dir = std::env::current_dir().map_err(Error::WorkingDirRead)?;
            let workspace_dir = match dest.workspace_spec() {
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
            };

            WorkspaceDir::new(workspace_dir)
        };

        let profile_dir = ProfileDir::new(workspace_dir.join(TARGET_DIR).join(&*profile));

        train_report.insert(profile);
        train_report.insert(profile_dir);
        train_report.insert(workspace_dir);

        Ok(())
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

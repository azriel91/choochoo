use std::{fs::File, io::BufWriter, marker::PhantomData};

use choochoo_cfg_model::{rt::ResIds, StationId};
use choochoo_rt_model::{Error, ProfileHistoryStationDir};

/// Persists resource IDs produced by stations into the profile history
/// directory.
///
/// The path to each resource ID file is:
///
/// ```text
/// ${workspace}/target/.history/${profile}/${station_id}/${res_id_logical}
/// ```
///
/// This is intended to record the resources created during each execution to
/// help discovery of what resources exist.
#[derive(Debug)]
pub struct ResIdPersister<E>(PhantomData<E>);

impl<E> ResIdPersister<E>
where
    E: Send + Sync + 'static,
{
    /// Persists resource IDs produced by stations into the profile history
    /// directory.
    ///
    /// The path to each resource ID file is:
    ///
    /// ```text
    /// ${workspace}/target/.history/${profile}/${station_id}/${res_id_logical}
    /// ```
    pub async fn persist(
        profile_history_station_dir: &ProfileHistoryStationDir,
        station_id: &StationId,
        res_ids: &ResIds,
    ) -> Result<(), Error<E>> {
        res_ids
            .iter()
            .try_for_each(|(res_id_logical, res_id_physical)| {
                let mut res_id_path = profile_history_station_dir.join(res_id_logical.as_str());
                res_id_path.set_extension("json");

                let res_id_path = File::create(&res_id_path).map_err(|error| {
                    let station_id = station_id.clone();
                    Error::<E>::ResIdWrite { station_id, error }
                })?;
                let writer = BufWriter::new(res_id_path);
                serde_json::to_writer_pretty(writer, res_id_physical).map_err(|error| {
                    let station_id = station_id.clone();
                    Error::ResIdSerialize { station_id, error }
                })
            })
    }
}

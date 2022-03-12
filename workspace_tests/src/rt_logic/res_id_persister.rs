use std::path::Path;

use choochoo_cfg_model::{
    rt::{ResIdLogical, ResIds, StationRtId, TrainResources},
    StationSpec,
};
use choochoo_resource::Profile;
use choochoo_rt_logic::{ResIdPersister, ResourceInitializer};
use choochoo_rt_model::{Destination, ProfileHistoryStationDirs, WorkspaceSpec};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use tokio::runtime;

#[test]
fn writes_res_ids_in_profile_history_dir() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(async {
        let (_tempdir, dest, train_resources, station_rt_id) = setup().await?;

        let profile_history_station_dirs = train_resources.borrow::<ProfileHistoryStationDirs>();
        let profile_history_station_dir = &profile_history_station_dirs[&station_rt_id];
        let station_id = dest.station_specs()[station_rt_id].id();
        let mut res_ids = ResIds::new();
        res_ids.insert(ResIdLogical::new("res_a"), ResA(123));
        res_ids.insert(
            ResIdLogical::new("res_b"),
            ResB {
                value: "a string".to_string(),
            },
        );
        ResIdPersister::<()>::persist(&profile_history_station_dir, &station_id, &res_ids).await?;

        let res_a_serialized =
            tokio::fs::read_to_string(profile_history_station_dir.join("res_a.json")).await?;
        assert_eq!("123", res_a_serialized);

        let res_b_serialized =
            tokio::fs::read_to_string(profile_history_station_dir.join("res_b.json")).await?;
        assert_eq!(
            r#"{
  "value": "a string"
}"#,
            res_b_serialized
        );

        Result::<_, Box<dyn std::error::Error>>::Ok(())
    })
}

async fn setup()
-> Result<(TempDir, Destination<()>, TrainResources<()>, StationRtId), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let mut dest_builder = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf()))
        .with_profile(Profile::new("profile")?);
    let station_rt_id = dest_builder.add_station(StationSpec::mock("station_a")?.build());
    let dest = dest_builder.build()?;
    let mut train_resources = TrainResources::new();
    ResourceInitializer::initialize(&dest, &mut train_resources).await?;
    Ok((tempdir, dest, train_resources, station_rt_id))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ResA(u32);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ResB {
    value: String,
}

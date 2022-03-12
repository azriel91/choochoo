use std::path::Path;

use choochoo_cfg_model::{fn_graph::FnId, rt::TrainResources, StationSpec};
use choochoo_resource::{HistoryDir, Profile, ProfileDir, ProfileHistoryDir, WorkspaceDir};
use choochoo_rt_logic::ResourceInitializer;
use choochoo_rt_model::{Destination, ProfileHistoryStationDirs, StationDirs, WorkspaceSpec};
use tokio::runtime;

#[test]
fn inserts_workspace_dir() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf()))
        .build()?;
    let mut train_resources = TrainResources::new();

    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(ResourceInitializer::initialize(&dest, &mut train_resources))?;

    let workspace_dir = train_resources.borrow::<WorkspaceDir>();
    assert!(&**workspace_dir == tempdir.path());

    Ok(())
}

#[test]
fn inserts_profile() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf()))
        .with_profile(Profile::new("profile")?)
        .build()?;
    let mut train_resources = TrainResources::new();

    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(ResourceInitializer::initialize(&dest, &mut train_resources))?;

    assert_eq!("profile", &**train_resources.borrow::<Profile>());

    Ok(())
}

#[test]
fn inserts_history_dir() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf()))
        .with_profile(Profile::new("profile")?)
        .build()?;
    let mut train_resources = TrainResources::new();

    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(ResourceInitializer::initialize(&dest, &mut train_resources))?;

    let history_dir = train_resources.borrow::<HistoryDir>();
    assert!(
        history_dir.ends_with("target/.history"),
        "Expected history directory `{}` to end with `target/.history`",
        history_dir.display()
    );
    assert!(history_dir.exists());

    Ok(())
}

#[test]
fn inserts_profile_history_dir() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf()))
        .with_profile(Profile::new("profile")?)
        .build()?;
    let mut train_resources = TrainResources::new();

    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(ResourceInitializer::initialize(&dest, &mut train_resources))?;

    let profile_history_dir = train_resources.borrow::<ProfileHistoryDir>();
    assert!(
        profile_history_dir.ends_with("target/.history/profile"),
        "Expected profile history directory `{}` to end with `target/.history/profile`",
        profile_history_dir.display()
    );
    assert!(profile_history_dir.exists());

    Ok(())
}

#[test]
fn inserts_profile_history_station_dirs() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let dest = {
        let mut dest_builder = Destination::<()>::builder()
            .with_workspace_spec(WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf()))
            .with_profile(Profile::new("profile")?);
        dest_builder.add_stations([
            StationSpec::mock("station_a")?.build(),
            StationSpec::mock("station_b")?.build(),
        ]);

        dest_builder.build()?
    };
    let mut train_resources = TrainResources::new();

    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(ResourceInitializer::initialize(&dest, &mut train_resources))?;

    let profile_history_station_dirs = train_resources.borrow::<ProfileHistoryStationDirs>();
    assert!(profile_history_station_dirs.iter().any(
        |(station_rt_id, profile_history_station_dir)| *station_rt_id == FnId::new(0)
            && profile_history_station_dir.ends_with("target/.history/profile/station_a")
    ));
    assert!(profile_history_station_dirs.iter().any(
        |(station_rt_id, profile_history_station_dir)| *station_rt_id == FnId::new(1)
            && profile_history_station_dir.ends_with("target/.history/profile/station_b")
    ));
    assert!(
        profile_history_station_dirs
            .values()
            .all(|profile_history_station_dir| profile_history_station_dir.exists())
    );

    Ok(())
}

#[test]
fn inserts_profile_dir() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf()))
        .with_profile(Profile::new("profile")?)
        .build()?;
    let mut train_resources = TrainResources::new();

    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(ResourceInitializer::initialize(&dest, &mut train_resources))?;

    let profile_dir = train_resources.borrow::<ProfileDir>();
    assert!(
        profile_dir.ends_with("target/profile"),
        "Expected profile directory `{}` to end with `target/profile`",
        profile_dir.display()
    );
    assert!(profile_dir.exists());

    Ok(())
}

#[test]
fn inserts_station_dirs() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let dest = {
        let mut dest_builder = Destination::<()>::builder()
            .with_workspace_spec(WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf()))
            .with_profile(Profile::new("profile")?);
        dest_builder.add_stations([
            StationSpec::mock("station_a")?.build(),
            StationSpec::mock("station_b")?.build(),
        ]);

        dest_builder.build()?
    };
    let mut train_resources = TrainResources::new();

    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(ResourceInitializer::initialize(&dest, &mut train_resources))?;

    let station_dirs = train_resources.borrow::<StationDirs>();
    assert!(station_dirs.iter().any(
        |(station_rt_id, station_dir)| *station_rt_id == FnId::new(0)
            && station_dir.ends_with("target/profile/station_a")
    ));
    assert!(station_dirs.iter().any(
        |(station_rt_id, station_dir)| *station_rt_id == FnId::new(1)
            && station_dir.ends_with("target/profile/station_b")
    ));
    assert!(
        station_dirs
            .values()
            .all(|station_dir| station_dir.exists())
    );

    Ok(())
}

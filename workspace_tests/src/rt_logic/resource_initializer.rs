use std::path::Path;

use choochoo_cfg_model::{fn_graph::FnId, rt::TrainResources, StationSpec};
use choochoo_resource::{Profile, ProfileDir, WorkspaceDir};
use choochoo_rt_logic::ResourceInitializer;
use choochoo_rt_model::{Destination, StationDirs, WorkspaceSpec};
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
    assert!(
        station_dirs
            .iter()
            .any(|(fn_id, station_dir)| *fn_id == FnId::new(0)
                && station_dir.ends_with("target/profile/station_a"))
    );
    assert!(
        station_dirs
            .iter()
            .any(|(fn_id, station_dir)| *fn_id == FnId::new(1)
                && station_dir.ends_with("target/profile/station_b"))
    );

    Ok(())
}

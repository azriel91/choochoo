use std::path::Path;

use choochoo_cfg_model::{fn_graph::FnId, rt::TrainReport, StationSpec};
use choochoo_resource::{Profile, ProfileDir, WorkspaceDir};
use choochoo_rt_logic::ResourceInitializer;
use choochoo_rt_model::{Destination, StationDirs, WorkspaceSpec};

#[test]
fn inserts_workspace_dir() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::WorkingDir)
        .build()?;
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    let workspace_dir = train_report.borrow::<WorkspaceDir>();
    assert!(
        workspace_dir.ends_with("choochoo/workspace_tests"),
        "Expected `{}` to end with `choochoo/workspace_tests`",
        workspace_dir.display()
    );

    Ok(())
}

#[test]
fn inserts_profile() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_profile(Profile::new("profile")?)
        .build()?;
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    assert_eq!("profile", &**train_report.borrow::<Profile>());

    Ok(())
}

#[test]
fn inserts_profile_dir() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_profile(Profile::new("profile")?)
        .with_workspace_spec(WorkspaceSpec::WorkingDir)
        .build()?;
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    let profile_dir = train_report.borrow::<ProfileDir>();
    assert!(
        profile_dir.ends_with("choochoo/workspace_tests/target/profile"),
        "Expected profile directory `{}` to end with `choochoo/workspace_tests/target/profile`",
        profile_dir.display()
    );

    Ok(())
}

#[test]
fn inserts_station_dirs() -> Result<(), Box<dyn std::error::Error>> {
    let dest = {
        let mut dest_builder = Destination::<()>::builder()
            .with_profile(Profile::new("profile")?)
            .with_workspace_spec(WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock")));
        dest_builder.add_stations([
            StationSpec::mock("station_a")?.build(),
            StationSpec::mock("station_b")?.build(),
        ]);

        dest_builder.build()?
    };
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    let station_dirs = train_report.borrow::<StationDirs>();
    assert!(
        station_dirs
            .iter()
            .any(|(fn_id, station_dir)| *fn_id == FnId::new(0)
                && station_dir.ends_with("choochoo/target/profile/station_a"))
    );
    assert!(
        station_dirs
            .iter()
            .any(|(fn_id, station_dir)| *fn_id == FnId::new(1)
                && station_dir.ends_with("choochoo/target/profile/station_b"))
    );

    Ok(())
}

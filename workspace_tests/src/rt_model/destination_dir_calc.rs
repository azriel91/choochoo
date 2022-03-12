use std::path::Path;

use choochoo_cfg_model::{
    fn_graph::{FnGraph, FnGraphBuilder, FnId},
    StationSpec, StationSpecs,
};
use choochoo_resource::Profile;
use choochoo_rt_model::{DestinationDirCalc, DestinationDirs, WorkspaceSpec};

#[test]
fn calculates_workspace_dir_from_working_directory() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::WorkingDir;
    let profile = Profile::default();
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs { workspace_dir, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        workspace_dir.ends_with("choochoo/workspace_tests"),
        "Expected `{}` to end with `choochoo/workspace_tests`",
        workspace_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_workspace_dir_from_first_dir_with_file() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock"));
    let profile = Profile::default();
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs { workspace_dir, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        workspace_dir.ends_with("choochoo"),
        "Expected `{}` to end with `choochoo`",
        workspace_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_workspace_dir_from_path() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace_spec = WorkspaceSpec::Path(Path::new(tempdir.path()).to_path_buf());
    let profile = Profile::default();
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs { workspace_dir, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(&*workspace_dir == tempdir.path());

    Ok(())
}

#[test]
fn calculates_history_dir_relative_to_workspace_dir() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock"));
    let profile = Profile::default();
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs { history_dir, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        history_dir.ends_with("choochoo/target/.history"),
        "Expected `{}` to end with `choochoo/target/.history`",
        history_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_profile_history_dir_from_first_dir_with_file_and_default_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock"));
    let profile = Profile::default();
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs {
        profile_history_dir,
        ..
    } = DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        profile_history_dir.ends_with("choochoo/target/.history/default"),
        "Expected `{}` to end with `choochoo/target/.history/default`",
        profile_history_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_profile_history_dir_from_first_dir_with_file_and_custom_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock"));
    let profile = Profile::new("custom")?;
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs {
        profile_history_dir,
        ..
    } = DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        profile_history_dir.ends_with("choochoo/target/.history/custom"),
        "Expected `{}` to end with `choochoo/target/.history/custom`",
        profile_history_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_profile_dir_from_working_directory_and_default_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::WorkingDir;
    let profile = Profile::default();
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs { profile_dir, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        profile_dir.ends_with(Path::new("choochoo/workspace_tests/target/default")),
        "Expected profile directory `{}` to end with `choochoo/workspace_tests/target/default`",
        profile_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_profile_dir_from_working_directory_and_custom_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::WorkingDir;
    let profile = Profile::new("custom")?;
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs { profile_dir, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        profile_dir.ends_with(Path::new("choochoo/workspace_tests/target/custom")),
        "Expected profile directory `{}` to end with `choochoo/workspace_tests/target/custom`",
        profile_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_profile_dir_from_first_dir_with_file_and_default_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock"));
    let profile = Profile::default();
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs { profile_dir, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        profile_dir.ends_with("choochoo/target/default"),
        "Expected `{}` to end with `choochoo/target/default`",
        profile_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_profile_dir_from_first_dir_with_file_and_custom_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock"));
    let profile = Profile::new("custom")?;
    let station_specs = StationSpecs::<()>::new(FnGraph::new());

    let DestinationDirs { profile_dir, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

    assert!(
        profile_dir.ends_with("choochoo/target/custom"),
        "Expected `{}` to end with `choochoo/target/custom`",
        profile_dir.display()
    );

    Ok(())
}

#[test]
fn calculates_station_dirs_from_station_id_and_workspace_dir()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_spec = WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock"));
    let profile = Profile::new("profile")?;
    let station_specs = {
        let mut station_specs_builder = FnGraphBuilder::new();
        station_specs_builder.add_fns([
            StationSpec::mock("station_a")?.build(),
            StationSpec::mock("station_b")?.build(),
        ]);
        StationSpecs::<()>::new(station_specs_builder.build())
    };

    let DestinationDirs { station_dirs, .. } =
        DestinationDirCalc::calc(&workspace_spec, &profile, &station_specs)?;

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

use std::path::Path;

use choochoo_cfg_model::rt::TrainReport;
use choochoo_resource::{Profile, ProfileDir, WorkspaceDir};
use choochoo_rt_logic::ResourceInitializer;
use choochoo_rt_model::{Destination, Error, WorkspaceSpec};

#[test]
fn inserts_default_profile() -> Result<(), Error<()>> {
    let dest = Destination::<()>::builder().build();
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    assert_eq!(Profile::DEFAULT_STR, &**train_report.borrow::<Profile>());

    Ok(())
}

#[test]
fn inserts_custom_profile() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_profile(Profile::new("custom")?)
        .build();
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    assert_eq!("custom", &**train_report.borrow::<Profile>());

    Ok(())
}

#[test]
fn inserts_workspace_dir_from_working_directory() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::WorkingDir)
        .build();
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
fn inserts_workspace_dir_from_first_dir_with_file() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock")))
        .build();
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    let workspace_dir = train_report.borrow::<WorkspaceDir>();
    assert!(
        workspace_dir.ends_with("choochoo"),
        "Expected `{}` to end with `choochoo`",
        workspace_dir.display()
    );

    Ok(())
}

#[test]
fn inserts_profile_dir_from_working_directory_and_default_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::WorkingDir)
        .build();
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    let profile_dir = train_report.borrow::<ProfileDir>();
    assert!(
        profile_dir.ends_with(Path::new("choochoo/workspace_tests/target/default")),
        "Expected profile directory `{}` to end with `choochoo/workspace_tests/target/default`",
        profile_dir.display()
    );

    Ok(())
}

#[test]
fn inserts_profile_dir_from_working_directory_and_custom_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_profile(Profile::new("custom")?)
        .with_workspace_spec(WorkspaceSpec::WorkingDir)
        .build();
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    let profile_dir = train_report.borrow::<ProfileDir>();
    assert!(
        profile_dir.ends_with(Path::new("choochoo/workspace_tests/target/custom")),
        "Expected profile directory `{}` to end with `choochoo/workspace_tests/target/custom`",
        profile_dir.display()
    );

    Ok(())
}

#[test]
fn inserts_profile_dir_from_first_dir_with_file_and_default_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_workspace_spec(WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock")))
        .build();
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    let profile_dir = train_report.borrow::<ProfileDir>();
    assert!(
        profile_dir.ends_with("choochoo/target/default"),
        "Expected `{}` to end with `choochoo/target/default`",
        profile_dir.display()
    );

    Ok(())
}

#[test]
fn inserts_profile_dir_from_first_dir_with_file_and_custom_profile()
-> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_profile(Profile::new("custom")?)
        .with_workspace_spec(WorkspaceSpec::FirstDirWithFile(&Path::new("Cargo.lock")))
        .build();
    let mut train_report = TrainReport::new();

    ResourceInitializer::initialize(&dest, &mut train_report)?;

    let profile_dir = train_report.borrow::<ProfileDir>();
    assert!(
        profile_dir.ends_with("choochoo/target/custom"),
        "Expected `{}` to end with `choochoo/target/custom`",
        profile_dir.display()
    );

    Ok(())
}

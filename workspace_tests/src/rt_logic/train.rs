use choochoo_cfg_model::{
    indexmap::IndexMap,
    rt::{CheckStatus, ProgressLimit, StationRtId, VisitStatus},
    SetupFn, StationFn, StationSpec,
};
use choochoo_rt_logic::Train;
use choochoo_rt_model::Destination;
use futures::future::FutureExt;
use tokio::runtime;

#[test]
fn reaches_empty_dest() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut dest = Destination::<()>::builder().build()?;

    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let station_errors = train_report.station_errors();
    assert!(station_errors.try_read()?.is_empty());
    Ok(())
}

#[test]
fn visits_all_stations_to_destination() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut dest = {
        let mut dest_builder = Destination::<()>::builder();
        dest_builder.add_station(
            StationSpec::mock("a")?
                .with_visit_fn(StationFn::ok(()))
                .build(),
        );
        dest_builder.add_station(
            StationSpec::mock("b")?
                .with_visit_fn(StationFn::ok(()))
                .build(),
        );
        dest_builder.build()?
    };
    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let station_errors = train_report.station_errors();
    assert!(station_errors.try_read()?.is_empty());
    assert!(dest.station_progresses().values().all(|station_progress| {
        station_progress.borrow().visit_status == VisitStatus::VisitSuccess
    }));

    Ok(())
}

#[test]
fn records_successful_and_failed_visits() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let station_a = dest_builder.add_station(
            StationSpec::mock("a")?
                .with_visit_fn(StationFn::ok(()))
                .build(),
        );
        let station_b = dest_builder.add_station(
            StationSpec::mock("b")?
                .with_visit_fn(StationFn::err(()))
                .build(),
        );
        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_b, ());
        errors
    };

    let station_errors = train_report.station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        VisitStatus::VisitSuccess,
        dest.station_progresses()[&station_a].borrow().visit_status
    );
    assert_eq!(
        VisitStatus::VisitFail,
        dest.station_progresses()[&station_b].borrow().visit_status
    );

    Ok(())
}

#[test]
fn records_check_fn_failure() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_visit_fn(StationFn::ok(()))
                .with_check_fn(StationFn::err(()))
                .build(),
            StationSpec::mock("b")?
                .with_visit_fn(StationFn::err(()))
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_a, ());
        // station b's err should not be reached, because station a failed.
        errors
    };

    let station_errors = train_report.station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        VisitStatus::CheckFail,
        dest.station_progresses()[&station_a].borrow().visit_status
    );
    assert_eq!(
        VisitStatus::ParentFail,
        dest.station_progresses()[&station_b].borrow().visit_status
    );

    Ok(())
}

#[test]
fn records_check_fn_failure_after_visit_success() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_setup_fn(SetupFn::new(|_, train_report| {
                    train_report.insert(0u32);
                    async { Ok(ProgressLimit::Steps(1)) }.boxed_local()
                }))
                .with_check_fn(StationFn::ok(CheckStatus::VisitRequired))
                .with_visit_fn(StationFn::ok(()))
                .build(),
            StationSpec::mock("b")?
                .with_visit_fn(StationFn::err(()))
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_a, ());
        errors.insert(station_b, ());
        errors
    };

    let station_errors = train_report.station_errors();
    assert_eq!(
        VisitStatus::VisitSuccess,
        dest.station_progresses()[&station_a].borrow().visit_status
    );
    assert_eq!(
        VisitStatus::VisitFail,
        dest.station_progresses()[&station_b].borrow().visit_status
    );
    assert_eq!(&errors_expected, &*station_errors.try_read()?);

    Ok(())
}

#[test]
fn sets_visit_unnecessary_if_nothing_changed() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_check_fn(StationFn::ok(CheckStatus::VisitNotRequired))
                .with_visit_fn(StationFn::ok(()))
                .build(),
            StationSpec::mock("b")?
                .with_check_fn(StationFn::ok(CheckStatus::VisitNotRequired))
                .with_visit_fn(StationFn::err(())) // proving this is never used
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let errors_expected = IndexMap::<StationRtId, ()>::new();

    let station_errors = train_report.station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        VisitStatus::VisitUnnecessary,
        dest.station_progresses()[&station_a].borrow().visit_status
    );
    assert_eq!(
        VisitStatus::VisitUnnecessary,
        dest.station_progresses()[&station_b].borrow().visit_status
    );

    Ok(())
}

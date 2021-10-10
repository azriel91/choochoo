use choochoo_cfg_model::{indexmap::IndexMap, StationFn, StationSpec, VisitStatus};
use choochoo_rt_logic::Train;
use choochoo_rt_model::Destination;
use tokio::runtime;

#[test]
fn reaches_empty_dest() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut dest = Destination::<()>::default();

    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let station_errors = train_report.station_errors();
    assert!(
        station_errors
            .try_read()
            .expect("Expected to read station_errors.")
            .is_empty()
    );
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
        dest_builder.build()
    };
    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let station_errors = train_report.station_errors();
    assert!(
        station_errors
            .try_read()
            .expect("Expected to read station_errors.")
            .is_empty()
    );
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
        let dest = dest_builder.build();

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::reach(&mut dest))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_b, ());
        errors
    };

    let station_errors = train_report.station_errors();
    assert_eq!(
        &errors_expected,
        &*station_errors
            .try_read()
            .expect("Expected to read station_errors.")
    );
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

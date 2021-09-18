use choochoo_cfg_model::{
    StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec, StationSpecFns,
    StationSpecs, VisitStatus,
};
use choochoo_rt_logic::Train;
use choochoo_rt_model::{indexmap::IndexMap, Destination, StationProgresses, StationRtId};
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
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::Queued,
            Ok(()),
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::Queued,
            Ok(()),
        )?;
        Destination::new(station_specs, station_progresses)
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
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::Queued,
            Ok(()),
        )?;
        let station_b = add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::Queued,
            Err(()),
        )?;
        let dest = Destination::new(station_specs, station_progresses);

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

fn add_station(
    station_specs: &mut StationSpecs<()>,
    station_progresses: &mut StationProgresses,
    station_id: &'static str,
    visit_status: VisitStatus,
    visit_result: Result<(), ()>,
) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
    let name = String::from(station_id);
    let station_id = StationId::new(station_id)?;
    let station_spec_fns = {
        let visit_fn = if visit_result.is_ok() {
            StationFn::new(|_, _| Box::pin(async move { Result::<(), ()>::Ok(()) }))
        } else {
            StationFn::new(|_, _| Box::pin(async move { Result::<(), ()>::Err(()) }))
        };
        StationSpecFns::new(visit_fn)
    };
    let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
    let station_progress = StationProgress::new(&station_spec, visit_status);
    let station_rt_id = station_specs.add_node(station_spec);

    station_progresses.insert(station_rt_id, station_progress);

    Ok(station_rt_id)
}

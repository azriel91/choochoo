use choochoo_cfg_model::{
    StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec, StationSpecFns,
    VisitStatus, Workload,
};
use choochoo_rt_logic::VisitStatusUpdater;
use choochoo_rt_model::{Destination, StationProgresses, StationRtId, StationSpecs};

#[test]
fn update_processes_all_possible_transitions() -> Result<(), Box<dyn std::error::Error>> {
    // a -> c
    //      ^
    // b --/
    //   \
    //    \--> d
    //
    // e --> f
    let mut station_specs = StationSpecs::new();
    let mut station_progresses = StationProgresses::new();
    let station_a = add_station(
        &mut station_specs,
        &mut station_progresses,
        "a",
        VisitStatus::VisitSuccess,
    )?;
    let station_b = add_station(
        &mut station_specs,
        &mut station_progresses,
        "b",
        VisitStatus::VisitSuccess,
    )?;
    let station_c = add_station(
        &mut station_specs,
        &mut station_progresses,
        "c",
        VisitStatus::NotReady,
    )?; // Should become `Queued`
    let station_d = add_station(
        &mut station_specs,
        &mut station_progresses,
        "d",
        VisitStatus::NotReady,
    )?; // Should become `Queued`
    let station_e = add_station(
        &mut station_specs,
        &mut station_progresses,
        "e",
        VisitStatus::VisitFail,
    )?;
    let station_f = add_station(
        &mut station_specs,
        &mut station_progresses,
        "f",
        VisitStatus::NotReady,
    )?; // Should become `ParentFail`
    station_specs.add_edge(station_a, station_c, Workload::default())?;
    station_specs.add_edge(station_b, station_c, Workload::default())?;
    station_specs.add_edge(station_b, station_d, Workload::default())?;
    station_specs.add_edge(station_e, station_f, Workload::default())?;
    let mut dest = Destination::new(station_specs, station_progresses);

    VisitStatusUpdater::update(&mut dest);

    let station_a = &dest.station_progresses()[&station_a];
    let station_b = &dest.station_progresses()[&station_b];
    let station_c = &dest.station_progresses()[&station_c];
    let station_d = &dest.station_progresses()[&station_d];
    let station_e = &dest.station_progresses()[&station_e];
    let station_f = &dest.station_progresses()[&station_f];
    assert_eq!(VisitStatus::VisitSuccess, station_a.borrow().visit_status);
    assert_eq!(VisitStatus::VisitSuccess, station_b.borrow().visit_status);
    assert_eq!(VisitStatus::Queued, station_c.borrow().visit_status);
    assert_eq!(VisitStatus::Queued, station_d.borrow().visit_status);
    assert_eq!(VisitStatus::VisitFail, station_e.borrow().visit_status);
    assert_eq!(VisitStatus::ParentFail, station_f.borrow().visit_status);
    Ok(())
}

#[test]
fn update_propagates_parent_fail_transitions() -> Result<(), Box<dyn std::error::Error>> {
    // a -> c -> d -> e
    //      ^
    // b --/
    let mut station_specs = StationSpecs::new();
    let mut station_progresses = StationProgresses::new();
    let station_a = add_station(
        &mut station_specs,
        &mut station_progresses,
        "a",
        VisitStatus::InProgress,
    )?;
    let station_b = add_station(
        &mut station_specs,
        &mut station_progresses,
        "b",
        VisitStatus::VisitFail,
    )?;
    let station_c = add_station(
        &mut station_specs,
        &mut station_progresses,
        "c",
        VisitStatus::NotReady,
    )?;
    let station_d = add_station(
        &mut station_specs,
        &mut station_progresses,
        "d",
        VisitStatus::NotReady,
    )?;
    let station_e = add_station(
        &mut station_specs,
        &mut station_progresses,
        "e",
        VisitStatus::NotReady,
    )?;
    station_specs.add_edge(station_a, station_c, Workload::default())?;
    station_specs.add_edge(station_b, station_c, Workload::default())?;
    station_specs.add_edge(station_c, station_d, Workload::default())?;
    station_specs.add_edge(station_d, station_e, Workload::default())?;
    let mut dest = Destination::new(station_specs, station_progresses);

    VisitStatusUpdater::update(&mut dest);

    let station_a = &dest.station_progresses()[&station_a];
    let station_b = &dest.station_progresses()[&station_b];
    let station_c = &dest.station_progresses()[&station_c];
    let station_d = &dest.station_progresses()[&station_d];
    let station_e = &dest.station_progresses()[&station_e];
    assert_eq!(VisitStatus::InProgress, station_a.borrow().visit_status);
    assert_eq!(VisitStatus::VisitFail, station_b.borrow().visit_status);
    assert_eq!(VisitStatus::ParentFail, station_c.borrow().visit_status);
    assert_eq!(VisitStatus::ParentFail, station_d.borrow().visit_status);
    assert_eq!(VisitStatus::ParentFail, station_e.borrow().visit_status);
    Ok(())
}

#[test]
fn updates_not_ready_to_queued_when_no_parents_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut station_specs = StationSpecs::new();
    let mut station_progresses = StationProgresses::new();
    let station_rt_id = add_station(
        &mut station_specs,
        &mut station_progresses,
        "n",
        VisitStatus::NotReady,
    )?;
    let dest = Destination::new(station_specs, station_progresses);

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_rt_id);

    assert_eq!(Some(VisitStatus::Queued), visit_status_next);
    Ok(())
}

#[test]
fn updates_not_ready_to_queued_when_all_parents_visit_success()
-> Result<(), Box<dyn std::error::Error>> {
    // a -> c
    //      ^
    // b --/
    let mut station_specs = StationSpecs::new();
    let mut station_progresses = StationProgresses::new();
    let station_a = add_station(
        &mut station_specs,
        &mut station_progresses,
        "a",
        VisitStatus::VisitSuccess,
    )?;
    let station_b = add_station(
        &mut station_specs,
        &mut station_progresses,
        "b",
        VisitStatus::VisitSuccess,
    )?;
    let station_c = add_station(
        &mut station_specs,
        &mut station_progresses,
        "c",
        VisitStatus::NotReady,
    )?;
    station_specs.add_edge(station_a, station_c, Workload::default())?;
    station_specs.add_edge(station_b, station_c, Workload::default())?;
    let dest = Destination::new(station_specs, station_progresses);

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

    assert_eq!(Some(VisitStatus::Queued), visit_status_next);
    Ok(())
}

#[test]
fn updates_not_ready_to_queued_when_all_parents_visit_success_or_unnecessary()
-> Result<(), Box<dyn std::error::Error>> {
    // a -> c
    //      ^
    // b --/
    let mut station_specs = StationSpecs::new();
    let mut station_progresses = StationProgresses::new();
    let station_a = add_station(
        &mut station_specs,
        &mut station_progresses,
        "a",
        VisitStatus::VisitSuccess,
    )?;
    let station_b = add_station(
        &mut station_specs,
        &mut station_progresses,
        "b",
        VisitStatus::VisitUnnecessary,
    )?;
    let station_c = add_station(
        &mut station_specs,
        &mut station_progresses,
        "c",
        VisitStatus::NotReady,
    )?;
    station_specs.add_edge(station_a, station_c, Workload::default())?;
    station_specs.add_edge(station_b, station_c, Workload::default())?;
    let dest = Destination::new(station_specs, station_progresses);

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

    assert_eq!(Some(VisitStatus::Queued), visit_status_next);
    Ok(())
}

#[test]
fn updates_not_ready_to_parent_fail_when_any_parents_visit_fail()
-> Result<(), Box<dyn std::error::Error>> {
    // a -> c
    //      ^
    // b --/
    let mut station_specs = StationSpecs::new();
    let mut station_progresses = StationProgresses::new();
    let station_a = add_station(
        &mut station_specs,
        &mut station_progresses,
        "a",
        VisitStatus::VisitSuccess,
    )?;
    let station_b = add_station(
        &mut station_specs,
        &mut station_progresses,
        "b",
        VisitStatus::VisitFail,
    )?;
    let station_c = add_station(
        &mut station_specs,
        &mut station_progresses,
        "c",
        VisitStatus::NotReady,
    )?;
    station_specs.add_edge(station_a, station_c, Workload::default())?;
    station_specs.add_edge(station_b, station_c, Workload::default())?;
    let dest = Destination::new(station_specs, station_progresses);

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

    assert_eq!(Some(VisitStatus::ParentFail), visit_status_next);
    Ok(())
}

#[test]
fn updates_not_ready_to_parent_fail_when_any_parents_parent_fail()
-> Result<(), Box<dyn std::error::Error>> {
    let mut station_specs = StationSpecs::new();
    let mut station_progresses = StationProgresses::new();
    let station_a = add_station(
        &mut station_specs,
        &mut station_progresses,
        "a",
        VisitStatus::VisitSuccess,
    )?;
    let station_b = add_station(
        &mut station_specs,
        &mut station_progresses,
        "b",
        VisitStatus::ParentFail,
    )?;
    let station_c = add_station(
        &mut station_specs,
        &mut station_progresses,
        "c",
        VisitStatus::NotReady,
    )?;
    station_specs.add_edge(station_a, station_c, Workload::default())?;
    station_specs.add_edge(station_b, station_c, Workload::default())?;
    let dest = Destination::new(station_specs, station_progresses);

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

    assert_eq!(Some(VisitStatus::ParentFail), visit_status_next);
    Ok(())
}

#[test]
fn no_change_to_not_ready_when_any_parents_on_other_status()
-> Result<(), Box<dyn std::error::Error>> {
    [
        VisitStatus::NotReady,
        VisitStatus::Queued,
        VisitStatus::InProgress,
    ]
    .iter()
    .copied()
    .try_for_each(|visit_status_parent| {
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::VisitSuccess,
        )?;
        let station_b = add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            visit_status_parent,
        )?;
        let station_c = add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::NotReady,
        )?;
        station_specs.add_edge(station_a, station_c, Workload::default())?;
        station_specs.add_edge(station_b, station_c, Workload::default())?;
        let dest = Destination::new(station_specs, station_progresses);

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

        assert_eq!(None, visit_status_next);

        Ok(())
    })
}

#[test]
fn no_change_to_parent_fail_visit_success_or_visit_fail() -> Result<(), Box<dyn std::error::Error>>
{
    [
        VisitStatus::ParentFail,
        VisitStatus::VisitSuccess,
        VisitStatus::VisitFail,
    ]
    .iter()
    .copied()
    .try_for_each(|visit_status| {
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        let station_a = add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            visit_status,
        )?;
        let dest = Destination::new(station_specs, station_progresses);

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_a);

        assert_eq!(None, visit_status_next);

        Ok(())
    })
}

fn add_station(
    station_specs: &mut StationSpecs<()>,
    station_progresses: &mut StationProgresses,
    station_id: &'static str,
    visit_status: VisitStatus,
) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
    let name = String::from(station_id);
    let station_id = StationId::new(station_id)?;
    let station_spec_fns = {
        let visit_fn = StationFn::new(|_, _| Box::pin(async { Result::<(), ()>::Ok(()) }));
        StationSpecFns::new(visit_fn)
    };
    let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
    let station_progress = StationProgress::new(&station_spec, visit_status);
    let station_rt_id = station_specs.add_node(station_spec);

    station_progresses.insert(station_rt_id, station_progress);

    Ok(station_rt_id)
}

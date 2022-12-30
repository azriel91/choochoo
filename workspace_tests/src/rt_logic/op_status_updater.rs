use choochoo_cfg_model::{rt::OpStatus, StationSpec};
use choochoo_rt_logic::OpStatusUpdater;
use choochoo_rt_model::Destination;

#[test]
fn update_processes_all_possible_transitions() -> Result<(), Box<dyn std::error::Error>> {
    // a -> c
    //      ^
    // b --/
    //   \
    //    \--> d
    //
    // e --> f
    let mut dest_builder = Destination::<()>::builder();
    let [
        station_a,
        station_b,
        station_c,
        station_d,
        station_e,
        station_f,
    ] = dest_builder.add_stations([
        StationSpec::mock("a")?.build(),
        StationSpec::mock("b")?.build(),
        StationSpec::mock("c")?.build(), // Should become `OpQueued`
        StationSpec::mock("d")?.build(), // Should become `OpQueued`
        StationSpec::mock("e")?.build(),
        StationSpec::mock("f")?.build(), // Should become `ParentFail`
    ]);
    dest_builder.add_edges([
        (station_a, station_c),
        (station_b, station_c),
        (station_b, station_d),
        (station_e, station_f),
    ])?;
    let mut dest = dest_builder.build()?;
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().op_status = OpStatus::WorkSuccess;
        station_progresses[&station_b].borrow_mut().op_status = OpStatus::WorkSuccess;
        station_progresses[&station_c].borrow_mut().op_status = OpStatus::ParentPending;
        station_progresses[&station_d].borrow_mut().op_status = OpStatus::ParentPending;
        station_progresses[&station_e].borrow_mut().op_status = OpStatus::WorkFail;
        station_progresses[&station_f].borrow_mut().op_status = OpStatus::ParentPending;
    }

    OpStatusUpdater::update(&mut dest);

    let station_progresses = dest.station_progresses();
    let station_a = &station_progresses[&station_a];
    let station_b = &station_progresses[&station_b];
    let station_c = &station_progresses[&station_c];
    let station_d = &station_progresses[&station_d];
    let station_e = &station_progresses[&station_e];
    let station_f = &station_progresses[&station_f];
    assert_eq!(OpStatus::WorkSuccess, station_a.borrow().op_status);
    assert_eq!(OpStatus::WorkSuccess, station_b.borrow().op_status);
    assert_eq!(OpStatus::OpQueued, station_c.borrow().op_status);
    assert_eq!(OpStatus::OpQueued, station_d.borrow().op_status);
    assert_eq!(OpStatus::WorkFail, station_e.borrow().op_status);
    assert_eq!(OpStatus::ParentFail, station_f.borrow().op_status);
    Ok(())
}

#[test]
fn update_propagates_parent_fail_transitions() -> Result<(), Box<dyn std::error::Error>> {
    // a -> c -> d -> e
    //      ^
    // b --/
    let mut dest_builder = Destination::<()>::builder();
    let [station_a, station_b, station_c, station_d, station_e] = dest_builder.add_stations([
        StationSpec::mock("a")?.build(),
        StationSpec::mock("b")?.build(),
        StationSpec::mock("c")?.build(),
        StationSpec::mock("d")?.build(),
        StationSpec::mock("e")?.build(),
    ]);
    dest_builder.add_edges([
        (station_a, station_c),
        (station_b, station_c),
        (station_c, station_d),
        (station_d, station_e),
    ])?;
    let mut dest = dest_builder.build()?;
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().op_status = OpStatus::WorkInProgress;
        station_progresses[&station_b].borrow_mut().op_status = OpStatus::WorkFail;
        station_progresses[&station_c].borrow_mut().op_status = OpStatus::ParentPending;
        station_progresses[&station_d].borrow_mut().op_status = OpStatus::ParentPending;
        station_progresses[&station_e].borrow_mut().op_status = OpStatus::ParentPending;
    }

    OpStatusUpdater::update(&mut dest);

    let station_a = &dest.station_progresses()[&station_a];
    let station_b = &dest.station_progresses()[&station_b];
    let station_c = &dest.station_progresses()[&station_c];
    let station_d = &dest.station_progresses()[&station_d];
    let station_e = &dest.station_progresses()[&station_e];
    assert_eq!(OpStatus::WorkInProgress, station_a.borrow().op_status);
    assert_eq!(OpStatus::WorkFail, station_b.borrow().op_status);
    assert_eq!(OpStatus::ParentFail, station_c.borrow().op_status);
    assert_eq!(OpStatus::ParentFail, station_d.borrow().op_status);
    assert_eq!(OpStatus::ParentFail, station_e.borrow().op_status);
    Ok(())
}

#[test]
fn updates_parent_pending_to_op_queued_when_no_parents_exist()
-> Result<(), Box<dyn std::error::Error>> {
    let mut dest_builder = Destination::<()>::builder();
    let station_a = dest_builder.add_station(StationSpec::mock("a")?.build());
    let mut dest = dest_builder.build()?;
    dest.station_progresses_mut()[&station_a]
        .borrow_mut()
        .op_status = OpStatus::ParentPending;

    let op_status_next = OpStatusUpdater::op_status_next(&dest, station_a);

    assert_eq!(Some(OpStatus::OpQueued), op_status_next);
    Ok(())
}

#[test]
fn updates_parent_pending_to_op_queued_when_all_parents_visit_success()
-> Result<(), Box<dyn std::error::Error>> {
    // a -> c
    //      ^
    // b --/
    let mut dest_builder = Destination::<()>::builder();
    let [station_a, station_b, station_c] = dest_builder.add_stations([
        StationSpec::mock("a")?.build(),
        StationSpec::mock("b")?.build(),
        StationSpec::mock("c")?.build(),
    ]);
    dest_builder.add_edges([(station_a, station_c), (station_b, station_c)])?;
    let mut dest = dest_builder.build()?;
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().op_status = OpStatus::WorkSuccess;
        station_progresses[&station_b].borrow_mut().op_status = OpStatus::WorkSuccess;
        station_progresses[&station_c].borrow_mut().op_status = OpStatus::ParentPending;
    }

    let op_status_next = OpStatusUpdater::op_status_next(&dest, station_c);

    assert_eq!(Some(OpStatus::OpQueued), op_status_next);
    Ok(())
}

#[test]
fn updates_parent_pending_to_op_queued_when_all_parents_visit_success_or_unnecessary()
-> Result<(), Box<dyn std::error::Error>> {
    // a -> c
    //      ^
    // b --/
    let mut dest_builder = Destination::<()>::builder();
    let [station_a, station_b, station_c] = dest_builder.add_stations([
        StationSpec::mock("a")?.build(),
        StationSpec::mock("b")?.build(),
        StationSpec::mock("c")?.build(),
    ]);
    dest_builder.add_edges([(station_a, station_c), (station_b, station_c)])?;
    let mut dest = dest_builder.build()?;
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().op_status = OpStatus::WorkSuccess;
        station_progresses[&station_b].borrow_mut().op_status = OpStatus::WorkUnnecessary;
        station_progresses[&station_c].borrow_mut().op_status = OpStatus::ParentPending;
    }

    let op_status_next = OpStatusUpdater::op_status_next(&dest, station_c);

    assert_eq!(Some(OpStatus::OpQueued), op_status_next);
    Ok(())
}

#[test]
fn updates_parent_pending_to_parent_fail_when_any_parents_visit_fail()
-> Result<(), Box<dyn std::error::Error>> {
    // a -> c
    //      ^
    // b --/
    let mut dest_builder = Destination::<()>::builder();
    let [station_a, station_b, station_c] = dest_builder.add_stations([
        StationSpec::mock("a")?.build(),
        StationSpec::mock("b")?.build(),
        StationSpec::mock("c")?.build(),
    ]);
    dest_builder.add_edges([(station_a, station_c), (station_b, station_c)])?;
    let mut dest = dest_builder.build()?;
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().op_status = OpStatus::WorkSuccess;
        station_progresses[&station_b].borrow_mut().op_status = OpStatus::WorkFail;
        station_progresses[&station_c].borrow_mut().op_status = OpStatus::ParentPending;
    }

    let op_status_next = OpStatusUpdater::op_status_next(&dest, station_c);

    assert_eq!(Some(OpStatus::ParentFail), op_status_next);
    Ok(())
}

#[test]
fn updates_parent_pending_to_parent_fail_when_any_parents_parent_fail()
-> Result<(), Box<dyn std::error::Error>> {
    let mut dest_builder = Destination::<()>::builder();
    let [station_a, station_b, station_c] = dest_builder.add_stations([
        StationSpec::mock("a")?.build(),
        StationSpec::mock("b")?.build(),
        StationSpec::mock("c")?.build(),
    ]);
    dest_builder.add_edges([(station_a, station_c), (station_b, station_c)])?;
    let mut dest = dest_builder.build()?;
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().op_status = OpStatus::WorkSuccess;
        station_progresses[&station_b].borrow_mut().op_status = OpStatus::ParentFail;
        station_progresses[&station_c].borrow_mut().op_status = OpStatus::ParentPending;
    }

    let op_status_next = OpStatusUpdater::op_status_next(&dest, station_c);

    assert_eq!(Some(OpStatus::ParentFail), op_status_next);
    Ok(())
}

#[test]
fn no_change_to_parent_pending_when_any_parents_on_other_status()
-> Result<(), Box<dyn std::error::Error>> {
    IntoIterator::into_iter([
        OpStatus::ParentPending,
        OpStatus::OpQueued,
        OpStatus::WorkInProgress,
    ])
    .try_for_each(|op_status_parent| {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b, station_c] = dest_builder.add_stations([
            StationSpec::mock("a")?.build(),
            StationSpec::mock("b")?.build(),
            StationSpec::mock("c")?.build(),
        ]);
        dest_builder.add_edges([(station_a, station_c), (station_b, station_c)])?;
        let mut dest = dest_builder.build()?;
        {
            let station_progresses = dest.station_progresses_mut();
            station_progresses[&station_a].borrow_mut().op_status = OpStatus::WorkSuccess;
            station_progresses[&station_b].borrow_mut().op_status = op_status_parent;
            station_progresses[&station_c].borrow_mut().op_status = OpStatus::ParentPending;
        }

        let op_status_next = OpStatusUpdater::op_status_next(&dest, station_c);

        assert_eq!(None, op_status_next);

        Ok(())
    })
}

#[test]
fn no_change_to_parent_fail_visit_success_or_visit_fail() -> Result<(), Box<dyn std::error::Error>>
{
    IntoIterator::into_iter([
        OpStatus::ParentFail,
        OpStatus::WorkSuccess,
        OpStatus::WorkFail,
    ])
    .try_for_each(|op_status| {
        let mut dest_builder = Destination::<()>::builder();
        let station_a = dest_builder.add_station(StationSpec::mock("a")?.build());
        let mut dest = dest_builder.build()?;
        dest.station_progresses_mut()[&station_a]
            .borrow_mut()
            .op_status = op_status;

        let op_status_next = OpStatusUpdater::op_status_next(&dest, station_a);

        assert_eq!(None, op_status_next);

        Ok(())
    })
}

#[test]
fn no_change_to_setup_queued_when_parents_on_setup_queued_or_setup_success()
-> Result<(), Box<dyn std::error::Error>> {
    IntoIterator::into_iter([OpStatus::SetupQueued, OpStatus::SetupSuccess]).try_for_each(
        |op_status_parent| {
            let mut dest_builder = Destination::<()>::builder();
            let [station_a, station_b] = dest_builder.add_stations([
                StationSpec::mock("a")?.build(),
                StationSpec::mock("b")?.build(),
            ]);
            dest_builder.add_edge(station_a, station_b)?;
            let mut dest = dest_builder.build()?;
            {
                let station_progresses = dest.station_progresses_mut();
                station_progresses[&station_a].borrow_mut().op_status = op_status_parent;
                station_progresses[&station_b].borrow_mut().op_status = OpStatus::SetupQueued;
            }

            let op_status_next = OpStatusUpdater::op_status_next(&dest, station_b);

            assert_eq!(None, op_status_next);

            Ok(())
        },
    )
}

#[test]
fn updates_setup_queued_to_parent_fail_when_parents_on_setup_fail_or_parent_fail()
-> Result<(), Box<dyn std::error::Error>> {
    IntoIterator::into_iter([OpStatus::SetupFail, OpStatus::ParentFail]).try_for_each(
        |op_status_parent| {
            let mut dest_builder = Destination::<()>::builder();
            let [station_a, station_b] = dest_builder.add_stations([
                StationSpec::mock("a")?.build(),
                StationSpec::mock("b")?.build(),
            ]);
            dest_builder.add_edge(station_a, station_b)?;
            let mut dest = dest_builder.build()?;
            {
                let station_progresses = dest.station_progresses_mut();
                station_progresses[&station_a].borrow_mut().op_status = op_status_parent;
                station_progresses[&station_b].borrow_mut().op_status = OpStatus::SetupQueued;
            }

            let op_status_next = OpStatusUpdater::op_status_next(&dest, station_b);

            assert_eq!(Some(OpStatus::ParentFail), op_status_next);

            Ok(())
        },
    )
}

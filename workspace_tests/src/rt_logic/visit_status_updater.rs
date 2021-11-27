use choochoo_cfg_model::{fn_graph::Edge, rt::VisitStatus, StationSpec};
use choochoo_rt_logic::VisitStatusUpdater;
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
        StationSpec::mock("c")?.build(), // Should become `VisitQueued`
        StationSpec::mock("d")?.build(), // Should become `VisitQueued`
        StationSpec::mock("e")?.build(),
        StationSpec::mock("f")?.build(), // Should become `ParentFail`
    ]);
    dest_builder.add_edges([
        (station_a, station_c, Edge::Logic),
        (station_b, station_c, Edge::Logic),
        (station_b, station_d, Edge::Logic),
        (station_e, station_f, Edge::Logic),
    ])?;
    let mut dest = dest_builder.build();
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
        station_progresses[&station_d].borrow_mut().visit_status = VisitStatus::ParentPending;
        station_progresses[&station_e].borrow_mut().visit_status = VisitStatus::VisitFail;
        station_progresses[&station_f].borrow_mut().visit_status = VisitStatus::ParentPending;
    }

    VisitStatusUpdater::update(&mut dest);

    let station_progresses = dest.station_progresses();
    let station_a = &station_progresses[&station_a];
    let station_b = &station_progresses[&station_b];
    let station_c = &station_progresses[&station_c];
    let station_d = &station_progresses[&station_d];
    let station_e = &station_progresses[&station_e];
    let station_f = &station_progresses[&station_f];
    assert_eq!(VisitStatus::VisitSuccess, station_a.borrow().visit_status);
    assert_eq!(VisitStatus::VisitSuccess, station_b.borrow().visit_status);
    assert_eq!(VisitStatus::VisitQueued, station_c.borrow().visit_status);
    assert_eq!(VisitStatus::VisitQueued, station_d.borrow().visit_status);
    assert_eq!(VisitStatus::VisitFail, station_e.borrow().visit_status);
    assert_eq!(VisitStatus::ParentFail, station_f.borrow().visit_status);
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
        (station_a, station_c, Edge::Logic),
        (station_b, station_c, Edge::Logic),
        (station_c, station_d, Edge::Logic),
        (station_d, station_e, Edge::Logic),
    ])?;
    let mut dest = dest_builder.build();
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::InProgress;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::VisitFail;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
        station_progresses[&station_d].borrow_mut().visit_status = VisitStatus::ParentPending;
        station_progresses[&station_e].borrow_mut().visit_status = VisitStatus::ParentPending;
    }

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
fn updates_parent_pending_to_visit_queued_when_no_parents_exist()
-> Result<(), Box<dyn std::error::Error>> {
    let mut dest_builder = Destination::<()>::builder();
    let station_a = dest_builder.add_station(StationSpec::mock("a")?.build());
    let mut dest = dest_builder.build();
    dest.station_progresses_mut()[&station_a]
        .borrow_mut()
        .visit_status = VisitStatus::ParentPending;

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_a);

    assert_eq!(Some(VisitStatus::VisitQueued), visit_status_next);
    Ok(())
}

#[test]
fn updates_parent_pending_to_visit_queued_when_all_parents_visit_success()
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
    dest_builder.add_edges([
        (station_a, station_c, Edge::Logic),
        (station_b, station_c, Edge::Logic),
    ])?;
    let mut dest = dest_builder.build();
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
    }

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

    assert_eq!(Some(VisitStatus::VisitQueued), visit_status_next);
    Ok(())
}

#[test]
fn updates_parent_pending_to_visit_queued_when_all_parents_visit_success_or_unnecessary()
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
    dest_builder.add_edges([
        (station_a, station_c, Edge::Logic),
        (station_b, station_c, Edge::Logic),
    ])?;
    let mut dest = dest_builder.build();
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::VisitUnnecessary;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
    }

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

    assert_eq!(Some(VisitStatus::VisitQueued), visit_status_next);
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
    dest_builder.add_edges([
        (station_a, station_c, Edge::Logic),
        (station_b, station_c, Edge::Logic),
    ])?;
    let mut dest = dest_builder.build();
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::VisitFail;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
    }

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

    assert_eq!(Some(VisitStatus::ParentFail), visit_status_next);
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
    dest_builder.add_edges([
        (station_a, station_c, Edge::Logic),
        (station_b, station_c, Edge::Logic),
    ])?;
    let mut dest = dest_builder.build();
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::ParentFail;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
    }

    let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

    assert_eq!(Some(VisitStatus::ParentFail), visit_status_next);
    Ok(())
}

#[test]
fn no_change_to_parent_pending_when_any_parents_on_other_status()
-> Result<(), Box<dyn std::error::Error>> {
    IntoIterator::into_iter([
        VisitStatus::ParentPending,
        VisitStatus::VisitQueued,
        VisitStatus::InProgress,
    ])
    .try_for_each(|visit_status_parent| {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b, station_c] = dest_builder.add_stations([
            StationSpec::mock("a")?.build(),
            StationSpec::mock("b")?.build(),
            StationSpec::mock("c")?.build(),
        ]);
        dest_builder.add_edges([
            (station_a, station_c, Edge::Logic),
            (station_b, station_c, Edge::Logic),
        ])?;
        let mut dest = dest_builder.build();
        {
            let station_progresses = dest.station_progresses_mut();
            station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::VisitSuccess;
            station_progresses[&station_b].borrow_mut().visit_status = visit_status_parent;
            station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
        }

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_c);

        assert_eq!(None, visit_status_next);

        Ok(())
    })
}

#[test]
fn no_change_to_parent_fail_visit_success_or_visit_fail() -> Result<(), Box<dyn std::error::Error>>
{
    IntoIterator::into_iter([
        VisitStatus::ParentFail,
        VisitStatus::VisitSuccess,
        VisitStatus::VisitFail,
    ])
    .try_for_each(|visit_status| {
        let mut dest_builder = Destination::<()>::builder();
        let station_a = dest_builder.add_station(StationSpec::mock("a")?.build());
        let mut dest = dest_builder.build();
        dest.station_progresses_mut()[&station_a]
            .borrow_mut()
            .visit_status = visit_status;

        let visit_status_next = VisitStatusUpdater::visit_status_next(&dest, station_a);

        assert_eq!(None, visit_status_next);

        Ok(())
    })
}

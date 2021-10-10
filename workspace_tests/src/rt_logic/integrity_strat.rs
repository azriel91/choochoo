use choochoo_cfg_model::{
    resman::Resources,
    rt::{ProgressLimit, TrainReport, VisitStatus},
    SetupFn, StationFn, StationIdInvalidFmt, StationSpec, StationSpecFns, Workload,
};
use choochoo_rt_logic::strategy::IntegrityStrat;
use choochoo_rt_model::{Destination, Error};
use tokio::{
    runtime,
    sync::mpsc::{self, Receiver, Sender},
};

#[test]
fn returns_empty_stream_when_no_stations_exist() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::default();

    let (call_count, stations_sequence) = call_iter(dest, None)?;

    assert_eq!(0, call_count);
    assert!(stations_sequence.is_empty());
    Ok(())
}

#[test]
fn returns_empty_stream_when_station_all_visit_success_or_failed()
-> Result<(), Box<dyn std::error::Error>> {
    let (tx, _rx) = mpsc::channel(10);
    let (mut dest, [station_a, station_b, station_c]) = {
        let mut dest_builder = Destination::<()>::builder();
        let station_ids = dest_builder.add_stations([
            station("a", Ok((tx, 0)))?,
            station("b", Err(()))?,
            station("c", Err(()))?,
        ]);
        (dest_builder.build(), station_ids)
    };
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::VisitFail;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentFail;
    }

    let (call_count, stations_sequence) = call_iter(dest, None)?;

    assert_eq!(0, call_count);
    assert!(stations_sequence.is_empty());
    Ok(())
}

#[test]
fn returns_visit_queued_stations_and_propagates_visit_queued()
-> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(10);
    let (mut dest, [station_a, station_b, station_c]) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b, station_c] = dest_builder.add_stations([
            station("a", Ok((tx.clone(), 0)))?,
            station("b", Ok((tx.clone(), 1)))?,
            station("c", Ok((tx, 2)))?,
        ]);
        dest_builder.add_edges([
            (station_a, station_c, Workload::default()),
            (station_b, station_c, Workload::default()),
        ])?;
        (dest_builder.build(), [station_a, station_b, station_c])
    };
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::VisitQueued;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::VisitQueued;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
    }

    let (call_count, stations_sequence) = call_iter(dest, Some(rx))?;

    assert_eq!(3, call_count);
    assert_eq!(vec![0u8, 1u8, 2u8], stations_sequence);
    Ok(())
}

fn station(
    station_id: &'static str,
    visit_result: Result<(Sender<u8>, u8), ()>,
) -> Result<StationSpec<()>, StationIdInvalidFmt<'static>> {
    let station_spec_fns = {
        let visit_fn = match visit_result {
            Ok((tx, n)) => StationFn::new(move |station, _| {
                let tx = tx.clone();
                Box::pin(async move {
                    station.progress.visit_status = VisitStatus::VisitSuccess;
                    tx.send(n).await.map_err(|_| ())
                })
            }),
            Err(_) => StationFn::err(()),
        };
        StationSpecFns::new(SetupFn::ok(ProgressLimit::Steps(10)), visit_fn)
    };
    let station_spec = StationSpec::mock(station_id)?
        .with_station_spec_fns(station_spec_fns)
        .build();
    Ok(station_spec)
}

fn call_iter(
    mut dest: Destination<()>,
    rx: Option<Receiver<u8>>,
) -> Result<(u32, Vec<u8>), Box<dyn std::error::Error>> {
    let mut resources = Resources::default();
    resources.insert::<u32>(0);

    let rt = runtime::Builder::new_current_thread().build()?;
    let call_count_and_values = rt.block_on(async {
        let resources = IntegrityStrat::iter(&mut dest, resources, |station, resources| {
            Box::pin(async move {
                station
                    .spec
                    .visit(station, &TrainReport::default())
                    .await
                    .expect("Failed to visit station.");

                *resources.borrow_mut::<u32>() += 1;
                resources
            })
        })
        .await?;
        let call_count = *resources.borrow::<u32>();

        let mut received_values = Vec::new();
        if let Some(mut rx) = rx {
            // Prevent test from hanging.
            rx.close();

            while let Some(n) = rx.recv().await {
                received_values.push(n);
            }
        }

        Result::<_, Error<()>>::Ok((call_count, received_values))
    })?;

    Ok(call_count_and_values)
}

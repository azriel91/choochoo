use choochoo_cfg_model::{
    resman::Resources, StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec,
    StationSpecFns, VisitStatus,
};
use choochoo_rt_logic::strategy::IntegrityStrat;
use choochoo_rt_model::{Destination, Error, StationProgresses, StationSpecs};
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
    let dest = {
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::VisitSuccess,
            Ok((tx, 0)),
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::VisitFail,
            Err(()),
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::ParentFail,
            Err(()),
        )?;
        Destination::new(station_specs, station_progresses)
    };

    let (call_count, stations_sequence) = call_iter(dest, None)?;

    assert_eq!(0, call_count);
    assert!(stations_sequence.is_empty());
    Ok(())
}

#[test]
fn returns_queued_stations_and_propagates_queued() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(10);
    let dest = {
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            VisitStatus::Queued,
            Ok((tx.clone(), 0)),
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            VisitStatus::Queued,
            Ok((tx.clone(), 1)),
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            VisitStatus::Queued,
            Ok((tx, 2)),
        )?;
        Destination::new(station_specs, station_progresses)
    };

    let (_call_count, stations_sequence) = call_iter(dest, Some(rx))?;

    assert_eq!(vec![0u8, 1u8, 2u8], stations_sequence);
    Ok(())
}

fn add_station(
    station_specs: &mut StationSpecs<()>,
    station_progresses: &mut StationProgresses,
    station_id: &'static str,
    visit_status: VisitStatus,
    visit_result: Result<(Sender<u8>, u8), ()>,
) -> Result<(), StationIdInvalidFmt<'static>> {
    let station_spec_fns = {
        let visit_fn = match visit_result {
            Ok((tx, n)) => StationFn::new(move |station_progress, _| {
                let tx = tx.clone();
                Box::pin(async move {
                    station_progress.visit_status = VisitStatus::VisitSuccess;
                    tx.send(n).await.map_err(|_| ())
                })
            }),
            Err(_) => StationFn::new(|_, _| Box::pin(async { Err(()) })),
        };
        StationSpecFns::new(visit_fn)
    };
    let name = String::from(station_id);
    let station_id = StationId::new(station_id)?;
    let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
    let station_progress = StationProgress::new(&station_spec, visit_status);
    let station_rt_id = station_specs.add_node(station_spec);
    station_progresses.insert(station_rt_id, station_progress);
    Ok(())
}

fn call_iter(
    mut dest: Destination<()>,
    rx: Option<Receiver<u8>>,
) -> Result<(u32, Vec<u8>), Box<dyn std::error::Error>> {
    let mut resources = Resources::default();
    resources.insert::<u32>(0);

    let rt = runtime::Builder::new_current_thread().build()?;
    let call_count_and_values = rt.block_on(async {
        let resources = IntegrityStrat::iter(&mut dest, resources, |_, station, resources| {
            Box::pin(async move {
                station
                    .spec
                    .visit(&mut station.progress, &Resources::default())
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

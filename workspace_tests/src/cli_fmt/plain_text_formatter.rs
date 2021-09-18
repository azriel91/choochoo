use tokio::runtime;

use choochoo_cfg_model::{
    StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec, StationSpecFns,
    StationSpecs, VisitStatus,
};
use choochoo_cli_fmt::PlainTextFormatter;
use choochoo_rt_model::{Destination, StationProgresses, StationRtId, TrainReport};

#[test]
fn writes_station_status_name_and_description() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut output = Vec::with_capacity(1024);
    let dest = {
        let mut station_specs = StationSpecs::new();
        let mut station_progresses = StationProgresses::new();
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "a",
            "A",
            "a_desc",
            VisitStatus::NotReady,
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "b",
            "B",
            "b_desc",
            VisitStatus::ParentFail,
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "c",
            "C",
            "c_desc",
            VisitStatus::Queued,
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "d",
            "D",
            "d_desc",
            VisitStatus::InProgress,
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "e",
            "E",
            "e_desc",
            VisitStatus::VisitSuccess,
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "f",
            "F",
            "f_desc",
            VisitStatus::VisitUnnecessary,
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "g",
            "G",
            "g_desc",
            VisitStatus::VisitFail,
        )?;
        add_station(
            &mut station_specs,
            &mut station_progresses,
            "h",
            "H",
            "h_desc",
            VisitStatus::CheckFail,
        )?;
        Destination::new(station_specs, station_progresses)
    };
    let train_report = TrainReport::new();

    rt.block_on(PlainTextFormatter::fmt(&mut output, &dest, &train_report))?;

    assert_eq!(
        "\
        ⏰ A: a_desc\n\
        ☠️ B: b_desc\n\
        ⏳ C: c_desc\n\
        ⏳ D: d_desc\n\
        ✅ E: e_desc\n\
        ✅ F: f_desc\n\
        ❌ G: g_desc\n\
        ❌ H: h_desc\n\
        ",
        String::from_utf8(output)?
    );

    Ok(())
}

fn add_station(
    station_specs: &mut StationSpecs<()>,
    station_progresses: &mut StationProgresses,
    station_id: &'static str,
    station_name: &'static str,
    station_desc: &'static str,
    visit_status: VisitStatus,
) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
    let station_id = StationId::new(station_id)?;
    let station_spec_fns = {
        let visit_fn = StationFn::new(|_, _| Box::pin(async { Result::<(), ()>::Ok(()) }));
        StationSpecFns::new(visit_fn)
    };
    let station_spec = StationSpec::new(
        station_id,
        String::from(station_name),
        String::from(station_desc),
        station_spec_fns,
    );
    let station_progress = StationProgress::new(&station_spec, visit_status);
    let station_rt_id = station_specs.add_node(station_spec);
    station_progresses.insert(station_rt_id, station_progress);

    Ok(station_rt_id)
}

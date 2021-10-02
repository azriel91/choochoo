use choochoo_cfg_model::{
    ProgressLimit, StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec,
    StationSpecFns, VisitStatus,
};

#[test]
fn display_returns_readable_informative_message() -> Result<(), StationIdInvalidFmt<'static>> {
    let station_id = StationId::new("station_id")?;
    let name = String::from("Station Name");
    let description = String::from("One liner.");
    let station_spec_fns = {
        let visit_fn = StationFn::new(|_, _| Box::pin(async { Result::<(), ()>::Ok(()) }));
        StationSpecFns::new(visit_fn)
    };
    let station_spec = StationSpec::new(station_id, name, description, station_spec_fns);
    let mut station_progress = StationProgress::new(&station_spec, ProgressLimit::Unknown);
    station_progress.visit_status = VisitStatus::InProgress;

    assert_eq!(
        "[InProgress] Station Name: One liner.",
        station_progress.display(&station_spec).to_string()
    );
    Ok(())
}

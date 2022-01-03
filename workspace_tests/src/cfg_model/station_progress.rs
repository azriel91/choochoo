use choochoo_cfg_model::{
    rt::{ProgressLimit, StationProgress, VisitStatus},
    OpFns, SetupFn, StationFn, StationId, StationIdInvalidFmt, StationSpec,
};

#[test]
fn display_returns_readable_informative_message() -> Result<(), StationIdInvalidFmt<'static>> {
    let station_id = StationId::new("station_id")?;
    let name = String::from("Station Name");
    let description = String::from("One liner.");
    let op_fns = { OpFns::<()>::new(SetupFn::ok(ProgressLimit::Unknown), StationFn::ok(())) };
    let station_spec = StationSpec::new(station_id, name, description, op_fns);
    let mut station_progress = StationProgress::new(&station_spec, ProgressLimit::Unknown);
    station_progress.visit_status = VisitStatus::InProgress;

    assert_eq!(
        "[InProgress] Station Name: One liner.",
        station_progress.display(&station_spec).to_string()
    );
    Ok(())
}

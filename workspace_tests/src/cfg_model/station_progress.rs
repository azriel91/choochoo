use choochoo_cfg_model::{
    rt::{OpStatus, ProgressLimit, ResourceIds, StationProgress},
    OpFns, SetupFn, StationFn, StationId, StationIdInvalidFmt, StationOp, StationSpec,
};

#[test]
fn display_returns_readable_informative_message() -> Result<(), StationIdInvalidFmt<'static>> {
    let station_id = StationId::new("station_id")?;
    let name = String::from("Station Name");
    let description = String::from("One liner.");
    let work_op_fns = OpFns::<ResourceIds, _, ()>::new(
        SetupFn::ok(ProgressLimit::Unknown),
        StationFn::ok(ResourceIds::new()),
    );
    let station_op = StationOp::new(work_op_fns, None);
    let station_spec = StationSpec::new(station_id, name, description, station_op);
    let mut station_progress = StationProgress::new(&station_spec, ProgressLimit::Unknown);
    station_progress.op_status = OpStatus::WorkInProgress;

    assert_eq!(
        "[WorkInProgress] Station Name: One liner.",
        station_progress.display(&station_spec).to_string()
    );
    Ok(())
}

use choochoo_cfg_model::{
    rt::ProgressLimit, SetupFn, StationFn, StationId, StationIdInvalidFmt, StationSpec,
    StationSpecFns,
};

#[test]
fn display_returns_readable_informative_message() -> Result<(), StationIdInvalidFmt<'static>> {
    let station_id = StationId::new("station_id")?;
    let name = String::from("Station Name");
    let description = String::from("One liner.");
    let station_spec_fns =
        { StationSpecFns::<()>::new(SetupFn::ok(ProgressLimit::Unknown), StationFn::ok(())) };
    let station_spec = StationSpec::new(station_id, name, description, station_spec_fns);

    assert_eq!("Station Name: One liner.", station_spec.to_string());
    Ok(())
}

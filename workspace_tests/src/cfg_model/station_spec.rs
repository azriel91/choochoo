use choochoo_cfg_model::{
    ProgressUnit, StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns,
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
    let station_spec = StationSpec::new(
        station_id,
        name,
        description,
        station_spec_fns,
        ProgressUnit::None,
    );

    assert_eq!("Station Name: One liner.", station_spec.to_string());
    Ok(())
}

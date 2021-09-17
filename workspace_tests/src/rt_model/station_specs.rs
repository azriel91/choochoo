use std::ops::{Deref, DerefMut};

use choochoo_cfg_model::{
    StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns, VisitStatus,
};
use choochoo_rt_model::{daggy::NodeIndex, StationRtId, StationSpecs};

#[test]
fn iter_with_indices_returns_iterator_with_all_stations() -> Result<(), StationIdInvalidFmt<'static>>
{
    let mut station_specs = StationSpecs::new();
    let a = add_station(&mut station_specs, "a")?;
    let b = add_station(&mut station_specs, "b")?;

    let indicies = station_specs
        .iter_with_indices()
        .map(|(node_index, _)| node_index)
        .collect::<Vec<NodeIndex>>();

    assert_eq!(vec![a, b], indicies);
    Ok(())
}

#[test]
fn deref() {
    let station_specs = StationSpecs::<()>::new();
    assert!(std::ptr::eq(Deref::deref(&station_specs), &station_specs.0));
}

#[test]
fn deref_mut() {
    let mut station_specs = StationSpecs::<()>::new();
    assert!(std::ptr::eq(
        DerefMut::deref_mut(&mut station_specs),
        &mut station_specs.0
    ));
}

fn add_station(
    station_specs: &mut StationSpecs<()>,
    station_id: &'static str,
) -> Result<StationRtId, StationIdInvalidFmt<'static>> {
    let name = String::from(station_id);
    let station_id = StationId::new(station_id)?;
    let station_spec_fns = {
        let visit_fn = StationFn::new(|station_progress, _| {
            Box::pin(async move {
                station_progress.visit_status = VisitStatus::VisitSuccess;
                Result::<(), ()>::Ok(())
            })
        });
        StationSpecFns::new(visit_fn)
    };
    let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
    Ok(station_specs.add_node(station_spec))
}

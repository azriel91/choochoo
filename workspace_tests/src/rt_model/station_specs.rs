use std::ops::{Deref, DerefMut};

use choochoo_cfg_model::{fn_graph::FnId, StationIdInvalidFmt, StationSpec, StationSpecs};

#[test]
fn iter_with_indices_returns_iterator_with_all_stations() -> Result<(), StationIdInvalidFmt<'static>>
{
    let mut station_specs = StationSpecs::default();
    let a = station_specs.add_node(StationSpec::<()>::mock("a")?.build());
    let b = station_specs.add_node(StationSpec::<()>::mock("b")?.build());

    let indicies = station_specs
        .iter_insertion_with_indices()
        .map(|(node_index, _)| node_index)
        .collect::<Vec<FnId>>();

    assert_eq!(vec![a, b], indicies);
    Ok(())
}

#[test]
fn deref() {
    let station_specs = StationSpecs::<()>::default();
    assert!(std::ptr::eq(Deref::deref(&station_specs), &station_specs.0));
}

#[test]
fn deref_mut() {
    let mut station_specs = StationSpecs::<()>::default();
    assert!(std::ptr::eq(
        DerefMut::deref_mut(&mut station_specs),
        &mut station_specs.0
    ));
}

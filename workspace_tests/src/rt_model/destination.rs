use choochoo_cfg_model::StationSpec;
use choochoo_rt_model::Destination;

#[test]
fn stations_iter_returns_stations_in_dependency_order() -> Result<(), Box<dyn std::error::Error>> {
    let (dest, [a, b, c, d, e, f]) = {
        let mut dest_builder = Destination::<()>::builder();
        let [a, b, c, d, e, f] = dest_builder.add_stations([
            StationSpec::mock("a")?.build(),
            StationSpec::mock("b")?.build(),
            StationSpec::mock("c")?.build(),
            StationSpec::mock("d")?.build(),
            StationSpec::mock("e")?.build(),
            StationSpec::mock("f")?.build(),
        ]);

        // c - e - a - d
        //  \    /   /
        //    b -   /
        //         /
        // f ------
        dest_builder.add_edges([(c, e), (c, b), (b, a), (e, a), (a, d), (f, d)])?;
        (dest_builder.build()?, [a, b, c, d, e, f])
    };

    let stations = dest
        .stations_iter()
        .map(|station| station.rt_id)
        .collect::<Vec<_>>();
    assert_eq!([f, c, e, b, a, d], stations.as_slice());

    Ok(())
}

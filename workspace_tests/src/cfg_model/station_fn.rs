use choochoo_cfg_model::StationFn;

#[test]
fn debug_impl_includes_all_fields() {
    let visit_fn = StationFn::<(), ()>::ok(());

    assert_eq!(
        "StationFn(fn(&'_ mut Station<R, E>) -> StationFnReturn<'_, E>)",
        format!("{:?}", visit_fn)
    );
}

#[test]
fn partial_eq_returns_true_for_same_instance() {
    let visit_fn = StationFn::<(), ()>::ok(());

    assert_eq!(&visit_fn, &visit_fn);
}

#[test]
fn partial_eq_returns_false_for_different_instance() {
    let visit_fn_0 = StationFn::<(), ()>::ok(());
    let visit_fn_1 = StationFn::<(), ()>::ok(());

    assert_ne!(&visit_fn_0, &visit_fn_1);
}

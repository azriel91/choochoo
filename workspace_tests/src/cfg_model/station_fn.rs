use choochoo_cfg_model::StationFn;

#[test]
fn debug_impl_includes_all_fields() {
    let work_fn = StationFn::<(), (), ()>::ok(());

    assert_eq!(
        "StationFn(fn(&'_ mut Station<R, RErr, E>) -> LocalBoxFuture<'_, Result<R, RErr>>)",
        format!("{:?}", work_fn)
    );
}

#[test]
fn partial_eq_returns_true_for_same_instance() {
    let work_fn = StationFn::<(), (), ()>::ok(());

    assert_eq!(&work_fn, &work_fn);
}

#[test]
fn partial_eq_returns_false_for_different_instance() {
    let work_fn_0 = StationFn::<(), (), ()>::ok(());
    let work_fn_1 = StationFn::<(), (), ()>::ok(());

    assert_ne!(&work_fn_0, &work_fn_1);
}

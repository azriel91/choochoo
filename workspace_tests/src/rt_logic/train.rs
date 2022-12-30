use choochoo_cfg_model::{
    indexmap::IndexMap,
    rt::{CheckStatus, OpStatus, ProgressLimit, ResIds, StationMutRef, StationRtId, VisitOp},
    CleanFns, SetupFn, StationFn, StationSpec,
};
use choochoo_rt_logic::Train;
use choochoo_rt_model::{error::StationSpecError, Destination};
use futures::future::{FutureExt, LocalBoxFuture};
use tokio::runtime;

#[test]
fn reach_create_reaches_empty_dest() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut dest = Destination::<()>::builder().build()?;

    let train_report = rt.block_on(Train::default().reach(&mut dest, VisitOp::Create))?;

    let station_errors = train_report.train_resources().station_errors();
    assert!(station_errors.try_read()?.is_empty());

    Ok(())
}

#[test]
fn reach_create_visits_all_stations_to_destination() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut dest = {
        let mut dest_builder = Destination::<()>::builder();
        dest_builder.add_station(
            StationSpec::mock("a")?
                .with_create_work_fn(StationFn::ok(ResIds::new()))
                .build(),
        );
        dest_builder.add_station(
            StationSpec::mock("b")?
                .with_create_work_fn(StationFn::ok(ResIds::new()))
                .build(),
        );
        dest_builder.build()?
    };
    let train_report = rt.block_on(Train::default().reach(&mut dest, VisitOp::Create))?;

    let station_errors = train_report.train_resources().station_errors();
    assert!(station_errors.try_read()?.is_empty());
    assert!(
        dest.station_progresses().values().all(|station_progress| {
            station_progress.borrow().op_status == OpStatus::WorkSuccess
        })
    );

    Ok(())
}

#[test]
fn reach_create_records_successful_and_failed_ops() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let station_a = dest_builder.add_station(
            StationSpec::mock("a")?
                .with_create_work_fn(StationFn::ok(ResIds::new()))
                .build(),
        );
        let station_b = dest_builder.add_station(
            StationSpec::mock("b")?
                .with_create_work_fn(StationFn::err((ResIds::new(), ())))
                .build(),
        );
        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::default().reach(&mut dest, VisitOp::Create))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_b, ());
        errors
    };

    let station_errors = train_report.train_resources().station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        OpStatus::WorkSuccess,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::WorkFail,
        dest.station_progresses()[&station_b].borrow().op_status
    );

    Ok(())
}

#[test]
fn reach_create_records_check_fn_failure() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_create_work_fn(StationFn::ok(ResIds::new()))
                .with_create_check_fn(StationFn::err(()))
                .build(),
            StationSpec::mock("b")?
                .with_create_work_fn(StationFn::err((ResIds::new(), ())))
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::default().reach(&mut dest, VisitOp::Create))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_a, ());
        // station b's err should not be reached, because station a failed.
        errors
    };

    let station_errors = train_report.train_resources().station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        OpStatus::CheckFail,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::ParentFail,
        dest.station_progresses()[&station_b].borrow().op_status
    );

    Ok(())
}

#[test]
fn reach_create_records_check_fn_failure_after_op_success() -> Result<(), Box<dyn std::error::Error>>
{
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_create_setup_fn(SetupFn::new(|_, train_resources| {
                    train_resources.insert(0u32);
                    async { Ok(ProgressLimit::Steps(1)) }.boxed_local()
                }))
                .with_create_check_fn(StationFn::ok(CheckStatus::WorkRequired))
                .with_create_work_fn(StationFn::ok(ResIds::new()))
                .build(),
            StationSpec::mock("b")?
                .with_create_work_fn(StationFn::err((ResIds::new(), ())))
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_resources = rt.block_on(Train::default().reach(&mut dest, VisitOp::Create))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_a, ());
        errors.insert(station_b, ());
        errors
    };

    let station_errors = train_resources.train_resources().station_errors();
    assert_eq!(
        OpStatus::WorkSuccess,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::WorkFail,
        dest.station_progresses()[&station_b].borrow().op_status
    );
    assert_eq!(&errors_expected, &*station_errors.try_read()?);

    Ok(())
}

#[test]
fn reach_create_sets_work_unnecessary_if_nothing_changed() -> Result<(), Box<dyn std::error::Error>>
{
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_create_check_fn(StationFn::ok(CheckStatus::WorkNotRequired))
                .with_create_work_fn(StationFn::ok(ResIds::new()))
                .build(),
            StationSpec::mock("b")?
                .with_create_check_fn(StationFn::ok(CheckStatus::WorkNotRequired))
                .with_create_work_fn(StationFn::err((ResIds::new(), ()))) // proving this is never used
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_resources = rt.block_on(Train::default().reach(&mut dest, VisitOp::Create))?;

    let errors_expected = IndexMap::<StationRtId, ()>::new();

    let station_errors = train_resources.train_resources().station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        OpStatus::WorkUnnecessary,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::WorkUnnecessary,
        dest.station_progresses()[&station_b].borrow().op_status
    );

    Ok(())
}

#[test]
fn reach_clean_reaches_empty_dest() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut dest = Destination::<()>::builder().build()?;

    let train_report = rt.block_on(Train::default().reach(&mut dest, VisitOp::Clean))?;

    let station_errors = train_report.train_resources().station_errors();
    assert!(station_errors.try_read()?.is_empty());

    Ok(())
}

#[test]
fn reach_clean_visits_all_stations_to_destination() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut dest = {
        let mut dest_builder = Destination::<()>::builder();
        dest_builder.add_station(
            StationSpec::mock("a")?
                .with_clean_fns(CleanFns::ok())
                .build(),
        );
        dest_builder.add_station(
            StationSpec::mock("b")?
                .with_clean_fns(CleanFns::ok())
                .build(),
        );
        dest_builder.build()?
    };
    let train_report = rt.block_on(Train::default().reach(&mut dest, VisitOp::Clean))?;

    let station_errors = train_report.train_resources().station_errors();
    assert!(station_errors.try_read()?.is_empty());
    assert!(
        dest.station_progresses().values().all(|station_progress| {
            station_progress.borrow().op_status == OpStatus::WorkSuccess
        }),
        "Expected all station progresses to be `OpStatus::WorkSuccess`, but they were: {station_progresses:?}",
        station_progresses = dest
            .station_progresses()
            .values()
            .map(|station_progress| { station_progress.borrow().op_status })
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[test]
fn reach_clean_records_successful_and_failed_ops() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let station_a = dest_builder.add_station(
            StationSpec::mock("a")?
                .with_clean_fns(CleanFns::ok())
                .build(),
        );
        let station_b = dest_builder.add_station(
            StationSpec::mock("b")?
                .with_clean_fns(CleanFns::err(()))
                .build(),
        );
        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::default().reach(&mut dest, VisitOp::Clean))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_b, ());
        errors
    };

    let station_errors = train_report.train_resources().station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        OpStatus::WorkSuccess,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::WorkFail,
        dest.station_progresses()[&station_b].borrow().op_status
    );

    Ok(())
}

#[test]
fn reach_clean_records_check_fn_failure() -> Result<(), Box<dyn std::error::Error>> {
    // Note:
    //
    // For this test, the create order is `a -> b`, so the clean order is `b -> a`.
    //
    // `b` is set up to err on the clean `check_fn`, so `a` should not be cleaned
    // due to `b`'s error.
    //
    // The `errors_expected` should then be a predecessor failure for `a`.

    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_clean_fns(CleanFns::err(()))
                .build(),
            StationSpec::mock("b")?
                .with_clean_fns(CleanFns::ok().with_check_fn(StationFn::err(())))
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_report = rt.block_on(Train::default().reach(&mut dest, VisitOp::Clean))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_b, ());
        // station b's err should not be reached, because station a failed.
        errors
    };

    let station_errors = train_report.train_resources().station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        OpStatus::ParentFail,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::CheckFail,
        dest.station_progresses()[&station_b].borrow().op_status
    );

    Ok(())
}

#[test]
fn reach_clean_records_check_fn_failure_after_op_success() -> Result<(), Box<dyn std::error::Error>>
{
    #[derive(Clone, Debug, PartialEq)]
    struct Error(u8);
    impl From<StationSpecError> for Error {
        fn from(_error: StationSpecError) -> Self {
            Error(1)
        }
    }

    fn b_clean_check<'f>(
        _: &'f mut StationMutRef<'_, Error>,
        n: &'f u32,
    ) -> LocalBoxFuture<'f, Result<CheckStatus, Error>> {
        async move {
            if *n == 0 {
                Ok(CheckStatus::WorkRequired)
            } else {
                Err(Error(2))
            }
        }
        .boxed_local()
    }

    fn b_clean_work<'f>(
        _: &'f mut StationMutRef<'_, Error>,
        n: &'f mut u32,
    ) -> LocalBoxFuture<'f, Result<(), Error>> {
        async move {
            *n += 1;
            Ok(())
        }
        .boxed_local()
    }

    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<Error>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_clean_fns(CleanFns::ok().with_check_fn(StationFn::err(Error(3))))
                .build(),
            StationSpec::mock("b")?
                .with_clean_fns(
                    CleanFns::new(
                        SetupFn::new(|_, train_resources| {
                            train_resources.insert(0u32);
                            async { Ok(ProgressLimit::Steps(1)) }.boxed_local()
                        }),
                        StationFn::new(b_clean_work),
                    )
                    .with_check_fn(StationFn::new(b_clean_check)),
                )
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_resources =
        rt.block_on(Train::<Error>::default().reach(&mut dest, VisitOp::Clean))?;

    let errors_expected = {
        let mut errors = IndexMap::new();
        errors.insert(station_b, Error(2));
        errors
    };

    let station_errors = train_resources.train_resources().station_errors();
    assert_eq!(
        OpStatus::ParentFail,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::CheckFail,
        dest.station_progresses()[&station_b].borrow().op_status
    );
    assert_eq!(&errors_expected, &*station_errors.try_read()?);

    Ok(())
}

#[test]
fn reach_clean_sets_work_unnecessary_if_nothing_changed() -> Result<(), Box<dyn std::error::Error>>
{
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_clean_fns(
                    CleanFns::ok().with_check_fn(StationFn::ok(CheckStatus::WorkNotRequired)),
                )
                .build(),
            StationSpec::mock("b")?
                .with_clean_fns(
                    CleanFns::ok().with_check_fn(StationFn::ok(CheckStatus::WorkNotRequired)),
                )
                .build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_resources = rt.block_on(Train::default().reach(&mut dest, VisitOp::Clean))?;

    let errors_expected = IndexMap::<StationRtId, ()>::new();

    let station_errors = train_resources.train_resources().station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        OpStatus::WorkUnnecessary,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::WorkUnnecessary,
        dest.station_progresses()[&station_b].borrow().op_status
    );

    Ok(())
}

#[test]
fn reach_clean_sets_work_unnecessary_if_clean_not_supported()
-> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let (mut dest, station_a, station_b) = {
        let mut dest_builder = Destination::<()>::builder();
        let [station_a, station_b] = dest_builder.add_stations([
            StationSpec::mock("a")?.build(),
            StationSpec::mock("b")?.build(),
        ]);
        dest_builder.add_edge(station_a, station_b)?;

        let dest = dest_builder.build()?;

        (dest, station_a, station_b)
    };
    let train_resources = rt.block_on(Train::default().reach(&mut dest, VisitOp::Clean))?;

    let errors_expected = IndexMap::<StationRtId, ()>::new();

    let station_errors = train_resources.train_resources().station_errors();
    assert_eq!(&errors_expected, &*station_errors.try_read()?);
    assert_eq!(
        OpStatus::WorkUnnecessary,
        dest.station_progresses()[&station_a].borrow().op_status
    );
    assert_eq!(
        OpStatus::WorkUnnecessary,
        dest.station_progresses()[&station_b].borrow().op_status
    );

    Ok(())
}

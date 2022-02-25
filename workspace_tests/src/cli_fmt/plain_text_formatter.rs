use tokio::runtime;

use choochoo_cfg_model::{
    rt::{OpStatus, StationErrors, StationRtId, TrainResources},
    StationSpec,
};
use choochoo_cli_fmt::PlainTextFormatter;
use choochoo_rt_model::Destination;

#[test]
fn writes_station_status_name_and_description() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut output = Vec::with_capacity(1024);
    let (
        mut dest,
        [
            station_a,
            station_b,
            station_c,
            station_d,
            station_e,
            station_f,
            station_g,
            station_h,
            station_i,
            station_j,
            station_k,
        ],
    ) = {
        let mut dest_builder = Destination::<()>::builder();
        let station_ids = dest_builder.add_stations([
            StationSpec::mock("a")?
                .with_name("A")
                .with_description("a_desc")
                .build(),
            StationSpec::mock("b")?
                .with_name("B")
                .with_description("b_desc")
                .build(),
            StationSpec::mock("c")?
                .with_name("C")
                .with_description("c_desc")
                .build(),
            StationSpec::mock("d")?
                .with_name("D")
                .with_description("d_desc")
                .build(),
            StationSpec::mock("e")?
                .with_name("E")
                .with_description("e_desc")
                .build(),
            StationSpec::mock("f")?
                .with_name("F")
                .with_description("f_desc")
                .build(),
            StationSpec::mock("g")?
                .with_name("G")
                .with_description("g_desc")
                .build(),
            StationSpec::mock("h")?
                .with_name("H")
                .with_description("h_desc")
                .build(),
            StationSpec::mock("i")?
                .with_name("I")
                .with_description("i_desc")
                .build(),
            StationSpec::mock("j")?
                .with_name("J")
                .with_description("j_desc")
                .build(),
            StationSpec::mock("k")?
                .with_name("K")
                .with_description("k_desc")
                .build(),
        ]);
        (dest_builder.build()?, station_ids)
    };
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().op_status = OpStatus::SetupQueued;
        station_progresses[&station_b].borrow_mut().op_status = OpStatus::SetupSuccess;
        station_progresses[&station_c].borrow_mut().op_status = OpStatus::SetupFail;
        station_progresses[&station_d].borrow_mut().op_status = OpStatus::ParentPending;
        station_progresses[&station_e].borrow_mut().op_status = OpStatus::ParentFail;
        station_progresses[&station_f].borrow_mut().op_status = OpStatus::OpQueued;
        station_progresses[&station_k].borrow_mut().op_status = OpStatus::CheckFail;
        station_progresses[&station_g].borrow_mut().op_status = OpStatus::WorkInProgress;
        station_progresses[&station_h].borrow_mut().op_status = OpStatus::WorkSuccess;
        station_progresses[&station_i].borrow_mut().op_status = OpStatus::WorkUnnecessary;
        station_progresses[&station_j].borrow_mut().op_status = OpStatus::WorkFail;
    }
    let train_resources = TrainResources::new();

    rt.block_on(PlainTextFormatter::fmt(
        &mut output,
        &dest,
        &train_resources,
    ))?;

    assert_eq!(
        "\
        ⏳ A: a_desc\n\
        ⏳ B: b_desc\n\
        ❌ C: c_desc\n\
        ⏰ D: d_desc\n\
        ☠️ E: e_desc\n\
        ⏳ F: f_desc\n\
        ⏳ G: g_desc\n\
        ✅ H: h_desc\n\
        ✅ I: i_desc\n\
        ❌ J: j_desc\n\
        ❌ K: k_desc\n\
        ",
        String::from_utf8(output)?
    );

    Ok(())
}

#[test]
fn formats_errors_as_human_readable_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = Vec::with_capacity(1024);
    let rt = runtime::Builder::new_current_thread().build()?;

    rt.block_on(async {
        let train_resources = TrainResources::<()>::new();
        {
            let errors = train_resources.borrow::<StationErrors<()>>();
            let mut errors = errors.write().await;
            errors.insert(StationRtId::new(0), ());
        }

        PlainTextFormatter::fmt_errors(&mut output, &train_resources).await
    })?;

    let output_expected = "\u{1b}[0m\u{1b}[1m\u{1b}[38;5;9merror\u{1b}[0m\u{1b}[1m: \u{1b}[0m\n\n";
    assert_eq!(
        output_expected,
        String::from_utf8(output).expect("Expected output to be valid UTF8.")
    );

    Ok(())
}

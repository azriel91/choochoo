use tokio::runtime;

use choochoo_cfg_model::{StationSpec, VisitStatus};
use choochoo_cli_fmt::PlainTextFormatter;
use choochoo_rt_model::{Destination, TrainReport};

#[test]
fn writes_station_status_name_and_description() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut output = Vec::with_capacity(1024);
    let (
        mut dest,
        [station_a, station_b, station_c, station_d, station_e, station_f, station_g, station_h, station_i, station_j],
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
        ]);
        (dest_builder.build(), station_ids)
    };
    {
        let station_progresses = dest.station_progresses_mut();
        station_progresses[&station_a].borrow_mut().visit_status = VisitStatus::SetupQueued;
        station_progresses[&station_b].borrow_mut().visit_status = VisitStatus::SetupFail;
        station_progresses[&station_c].borrow_mut().visit_status = VisitStatus::ParentPending;
        station_progresses[&station_d].borrow_mut().visit_status = VisitStatus::ParentFail;
        station_progresses[&station_e].borrow_mut().visit_status = VisitStatus::VisitQueued;
        station_progresses[&station_f].borrow_mut().visit_status = VisitStatus::InProgress;
        station_progresses[&station_g].borrow_mut().visit_status = VisitStatus::VisitSuccess;
        station_progresses[&station_h].borrow_mut().visit_status = VisitStatus::VisitUnnecessary;
        station_progresses[&station_i].borrow_mut().visit_status = VisitStatus::VisitFail;
        station_progresses[&station_j].borrow_mut().visit_status = VisitStatus::CheckFail;
    }
    let train_report = TrainReport::new();

    rt.block_on(PlainTextFormatter::fmt(&mut output, &dest, &train_report))?;

    assert_eq!(
        "\
        ⏳ A: a_desc\n\
        ❌ B: b_desc\n\
        ⏰ C: c_desc\n\
        ☠️ D: d_desc\n\
        ⏳ E: e_desc\n\
        ⏳ F: f_desc\n\
        ✅ G: g_desc\n\
        ✅ H: h_desc\n\
        ❌ I: i_desc\n\
        ❌ J: j_desc\n\
        ",
        String::from_utf8(output)?
    );

    Ok(())
}

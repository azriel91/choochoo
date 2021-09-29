use choochoo::{
    cfg_model::Workload, cli_fmt::PlainTextFormatter, rt_logic::Train, rt_model::Destination,
};
use tokio::runtime;

use crate::{
    demo_error::DemoError,
    dependency_mode::DependencyMode,
    error::{ErrorCode, ErrorDetail},
    station_a::StationA,
    station_b::StationB,
    station_c::StationC,
    station_d::StationD,
    station_e::StationE,
    station_f::StationF,
    station_g::StationG,
    station_h::StationH,
};

#[path = "demo/app_zip.rs"]
mod app_zip;
#[path = "demo/demo_error.rs"]
mod demo_error;
#[path = "demo/dependency_mode.rs"]
mod dependency_mode;
#[path = "demo/error.rs"]
mod error;
#[path = "demo/server_params.rs"]
mod server_params;
#[path = "demo/station_a.rs"]
mod station_a;
#[path = "demo/station_b.rs"]
mod station_b;
#[path = "demo/station_c.rs"]
mod station_c;
#[path = "demo/station_d.rs"]
mod station_d;
#[path = "demo/station_e.rs"]
mod station_e;
#[path = "demo/station_f.rs"]
mod station_f;
#[path = "demo/station_g.rs"]
mod station_g;
#[path = "demo/station_h.rs"]
mod station_h;
#[path = "demo/station_sleep.rs"]
mod station_sleep;

#[derive(Debug)]
pub struct Args {
    /// How task execution should be structured.
    pub dependency_mode: DependencyMode,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    let rt = runtime::Builder::new_multi_thread()
        .thread_name("choochoo-demo")
        .enable_io()
        .enable_time()
        .build()?;

    rt.block_on(async move {
        let mut dest = {
            let mut dest_builder = Destination::builder();

            // Formatting pending <https://github.com/rust-lang/rustfmt/issues/4530>
            let [
                station_a,
                station_b,
                station_c,
                station_d,
                station_e,
                station_f,
                station_g,
                station_h
            ] = dest_builder.add_stations([
                StationA::build()?,
                StationB::build()?,
                StationC::build()?,
                StationD::build()?,
                StationE::build()?,
                StationF::build()?,
                StationG::build()?,
                StationH::build()?,
            ]);

            if args.dependency_mode == DependencyMode::Sequential {
                dest_builder.add_edges([
                    (station_a, station_b, Workload::default()),
                    (station_b, station_c, Workload::default()),
                    (station_c, station_d, Workload::default()),
                    (station_d, station_e, Workload::default()),
                    (station_e, station_f, Workload::default()),
                    (station_f, station_g, Workload::default()),
                    (station_g, station_h, Workload::default()),
                ])?;
            } else {
                dest_builder.add_edges([
                    (station_a, station_b, Workload::default()),
                    (station_a, station_c, Workload::default()),
                    (station_b, station_e, Workload::default()),
                    (station_c, station_d, Workload::default()),
                    (station_d, station_e, Workload::default()),
                    (station_e, station_g, Workload::default()),
                    (station_f, station_g, Workload::default()),
                    (station_g, station_h, Workload::default()),
                ])?;
            }

            let dest = dest_builder.build();

            Result::<_, Box<dyn std::error::Error>>::Ok(dest)
        }?;
        let train_report = Train::reach(&mut dest).await?;

        let mut stdout = tokio::io::stdout();
        PlainTextFormatter::fmt_errors(&mut stdout, &train_report).await?;

        Result::<_, Box<dyn std::error::Error>>::Ok(())
    })?;

    Ok(())
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    let dependency_mode = if pargs.contains(["-c", "--concurrent"]) {
        DependencyMode::Concurrent
    } else {
        DependencyMode::Sequential
    };

    Ok(Args { dependency_mode })
}

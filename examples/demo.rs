use std::borrow::Cow;

use choochoo::{
    cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns, Workload},
    fmt::PlainTextFormatter,
    rt_model::{Destination, Station, Stations, VisitStatus},
    Train,
};

use daggy::{petgraph::graph::DefaultIx, NodeIndex};
use srcerr::SourceError;
use tokio::runtime;

use crate::{
    error::{ErrorCode, ErrorDetail},
    station_a::StationA,
    station_b::StationB,
    station_c::StationC,
};

#[path = "demo/app_zip.rs"]
mod app_zip;
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

type DemoError = SourceError<'static, ErrorCode, ErrorDetail, Files>;
type Files = srcerr::codespan::Files<Cow<'static, str>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;

    rt.block_on(async move {
        let mut dest = {
            let mut stations = Stations::new();
            let station_a = StationA::build(&mut stations)?;
            let station_b = StationB::build(&mut stations)?;
            let station_c = StationC::build(&mut stations)?;

            stations.add_edge(station_a, station_b, Workload::default())?;
            stations.add_edge(station_b, station_c, Workload::default())?;

            let dest = Destination { stations };

            Result::<_, Box<dyn std::error::Error>>::Ok(dest)
        }?;
        let train_report = Train::reach(&mut dest).await;

        let mut stdout = tokio::io::stdout();
        PlainTextFormatter::fmt(&mut stdout, &dest, &train_report).await?;

        Result::<_, Box<dyn std::error::Error>>::Ok(())
    })?;

    Ok(())
}

fn add_station<'files>(
    stations: &mut Stations<DemoError>,
    station_id: &'static str,
    station_name: &'static str,
    station_description: &'static str,
    visit_fn: StationFn<(), DemoError>,
) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
    let station_id = StationId::new(station_id)?;
    let station_name = String::from(station_name);
    let station_description = String::from(station_description);
    let station_spec_fns = StationSpecFns::new(visit_fn);
    let station_spec = StationSpec::new(
        station_id,
        station_name,
        station_description,
        station_spec_fns,
    );
    let station = Station::new(station_spec, VisitStatus::Queued);
    Ok(stations.add_node(station))
}

use std::borrow::Cow;

use choochoo::{
    cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns},
    fmt::PlainTextFormatter,
    rt_model::{Destination, Station, Stations, VisitStatus},
    Train,
};

use daggy::{petgraph::graph::DefaultIx, NodeIndex};

use srcerr::SourceError;
use tokio::runtime;

use crate::{
    error::{ErrorCode, ErrorDetail},
    station_a::station_a,
};

#[path = "demo/error.rs"]
mod error;
#[path = "demo/station_a.rs"]
mod station_a;

type ExampleError<'f> = SourceError<'f, ErrorCode, ErrorDetail, Files>;
type Files = codespan::Files<Cow<'static, str>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;

    rt.block_on(async move {
        let (mut dest, _station_a, _station_b) = {
            let mut stations = Stations::new();
            let station_a = station_a(&mut stations);
            let station_b = station_b(&mut stations);
            let dest = Destination { stations };

            (dest, station_a, station_b)
        };
        let train_report = Train::reach(&mut dest).await;

        let mut stdout = tokio::io::stdout();

        PlainTextFormatter::fmt(&mut stdout, &dest, &train_report)
            .await
            .expect("Failed to format train report.");
    });

    Ok(())
}

fn station_b(
    stations: &mut Stations<ExampleError<'_>>,
) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
    let visit_fn = StationFn::new(move |_station, _| {
        Box::pin(async move {
            // TODO: Create DB.
            Result::<(), ExampleError<'_>>::Ok(())
        })
    });
    add_station(
        stations,
        "b",
        "Create DB",
        "Creates the database for the web application.",
        visit_fn,
    )
}

fn add_station<'files>(
    stations: &mut Stations<ExampleError<'files>>,
    station_id: &'static str,
    station_name: &'static str,
    station_description: &'static str,
    visit_fn: StationFn<(), ExampleError<'files>>,
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

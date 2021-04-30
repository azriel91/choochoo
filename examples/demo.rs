use std::borrow::Cow;

use choochoo::{
    cfg_model::Workload,
    rt_model::{error::StationSpecError, Destination, Stations},
    Train,
};
use srcerr::{
    codespan_reporting::diagnostic::{Diagnostic, Severity},
    SourceError,
};
use tokio::runtime;

use crate::{
    error::{ErrorCode, ErrorDetail},
    station_a::StationA,
    station_b::StationB,
    station_c::StationC,
    station_d::StationD,
    station_e::StationE,
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
#[path = "demo/station_d.rs"]
mod station_d;
#[path = "demo/station_e.rs"]
mod station_e;

pub struct DemoError(pub SourceError<'static, ErrorCode, ErrorDetail, Files>);

impl DemoError {
    pub fn new(code: ErrorCode, detail: ErrorDetail, severity: Severity) -> Self {
        Self(SourceError::new(code, detail, severity))
    }
}

impl choochoo::rt_model::error::AsDiagnostic<'static> for DemoError {
    type Files = Files;

    fn as_diagnostic(
        &self,
        files: &Self::Files,
    ) -> Diagnostic<<Self::Files as srcerr::codespan_reporting::files::Files<'static>>::FileId>
    {
        SourceError::as_diagnostic(&self.0, files)
    }
}

impl From<StationSpecError> for DemoError {
    fn from(error: StationSpecError) -> DemoError {
        let code = ErrorCode::StationSpecError;
        let detail = ErrorDetail::StationSpecError(error);

        DemoError::new(code, detail, Severity::Bug)
    }
}

type Files = srcerr::codespan::Files<Cow<'static, str>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;

    rt.block_on(async move {
        let mut dest = {
            let mut stations = Stations::new();
            let station_a = stations.add_node(StationA::build()?);
            let station_b = stations.add_node(StationB::build()?);
            let station_c = stations.add_node(StationC::build()?);
            let station_d = stations.add_node(StationD::build()?);
            let station_e = stations.add_node(StationE::build()?);

            stations.add_edge(station_a, station_b, Workload::default())?;
            stations.add_edge(station_b, station_c, Workload::default())?;
            stations.add_edge(station_c, station_d, Workload::default())?;
            stations.add_edge(station_d, station_e, Workload::default())?;

            let dest = Destination { stations };

            Result::<_, Box<dyn std::error::Error>>::Ok(dest)
        }?;
        let _train_report = Train::reach(&mut dest).await;

        // let mut stdout = tokio::io::stdout();
        // PlainTextFormatter::fmt(&mut stdout, &dest, &train_report).await?;

        Result::<_, Box<dyn std::error::Error>>::Ok(())
    })?;

    Ok(())
}

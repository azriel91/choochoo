use choochoo::cfg_model::{StationId, StationIdInvalidFmt, StationOp, StationSpec};

use crate::DemoError;

pub(crate) use self::{
    station_a_clean::StationAClean, station_a_create::StationACreate,
    station_a_errors::StationAErrors,
};

#[path = "station_a/station_a_clean.rs"]
mod station_a_clean;
#[path = "station_a/station_a_create.rs"]
mod station_a_create;
#[path = "station_a/station_a_errors.rs"]
mod station_a_errors;

/// Download App
pub struct StationA;

impl StationA {
    /// Returns a station that uploads `app.zip` to a server.
    pub fn build() -> Result<StationSpec<DemoError>, StationIdInvalidFmt<'static>> {
        let create_fns = StationACreate::build();
        let clean_fns = StationAClean::build();
        let station_op = StationOp::new(create_fns, Some(clean_fns));

        let station_id = StationId::new("a")?;
        let station_name = String::from("Upload App");
        let station_description = String::from("Uploads web application to artifact server.");
        Ok(StationSpec::new(
            station_id,
            station_name,
            station_description,
            station_op,
        ))
    }
}

use std::{convert::TryFrom, fmt};

use resman::Resources;

use crate::{
    CheckStatus, ProgressUnit, StationFnReturn, StationId, StationIdInvalidFmt, StationProgress,
    StationSpecBuilder, StationSpecFns,
};

// **Note:** `Clone` is manually implemented to avoid the trait bound on `E`.
/// Behaviour specification of the station.
#[derive(Debug, PartialEq)]
pub struct StationSpec<E> {
    /// Unique identifier of the station.
    pub(crate) id: StationId,
    /// Human readable name of the station.
    pub(crate) name: String,
    /// Short description of the station's purpose.
    pub(crate) description: String,
    /// Steps to run when this station is visited.
    pub(crate) station_spec_fns: StationSpecFns<E>,
    /// Unit of measurement to display progress information.
    pub(crate) progress_unit: ProgressUnit,
}

impl<E> StationSpec<E> {
    /// Returns a new [`StationSpec`].
    ///
    /// You may prefer using the [`builder`] method to construct a
    /// `StationSpec`.
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    /// * `name`: Human readable name of the station.
    /// * `description`: Short description of the station's purpose.
    /// * `station_spec_fns`: Steps to run when this station is visited.
    /// * `progress_unit`: Unit of measurement to display progress information.
    ///
    /// [`builder`]: Self::builder
    pub fn new(
        id: StationId,
        name: String,
        description: String,
        station_spec_fns: StationSpecFns<E>,
        progress_unit: ProgressUnit,
    ) -> Self {
        Self {
            id,
            name,
            description,
            station_spec_fns,
            progress_unit,
        }
    }

    /// Returns a new [`StationSpecBuilder`].
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    /// * `station_spec_fns`: Steps to run when this station is visited.
    pub fn builder<Id>(
        id: Id,
        station_spec_fns: StationSpecFns<E>,
    ) -> Result<StationSpecBuilder<E>, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        StationSpecBuilder::new(id, station_spec_fns)
    }

    /// Returns a new [`StationSpecBuilder`] to build a mock [`StationSpec`].
    ///
    /// This defaults the [`StationSpecFns`] to be success / no-op functions.
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    #[cfg(feature = "mock")]
    pub fn mock<Id>(id: Id) -> Result<StationSpecBuilder<E>, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        StationSpecBuilder::mock(id)
    }

    /// Returns the unique identifier of the station.
    pub fn id(&self) -> &StationId {
        &self.id
    }

    /// Returns the human readable name of the station.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the short description of the station's purpose.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns this station's behaviours.
    pub fn station_spec_fns(&self) -> &StationSpecFns<E> {
        &self.station_spec_fns
    }

    /// Returns the unit of measurement used to display progress information.
    pub fn progress_unit(&self) -> ProgressUnit {
        self.progress_unit
    }

    /// Checks if the station needs to be visited.
    pub fn check<'f>(
        &self,
        station_progress: &'f mut StationProgress,
        resources: &'f Resources,
    ) -> Option<StationFnReturn<'f, CheckStatus, E>> {
        self.station_spec_fns
            .check_fn
            .clone()
            .map(move |check_fn| check_fn.0(station_progress, resources))
    }

    /// Returns a task to visit the station.
    pub fn visit<'f>(
        &self,
        station_progress: &'f mut StationProgress,
        resources: &'f Resources,
    ) -> StationFnReturn<'f, (), E> {
        let visit_fn = self.station_spec_fns.visit_fn.clone();
        visit_fn.0(station_progress, resources)
    }
}

impl<E> Clone for StationSpec<E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            station_spec_fns: self.station_spec_fns.clone(),
            progress_unit: self.progress_unit,
        }
    }
}

impl<E> fmt::Display for StationSpec<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

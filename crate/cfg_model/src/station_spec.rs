use std::{convert::TryFrom, fmt};

use fn_graph::{FnMeta, TypeIds};

use crate::{
    rt::{CheckStatus, StationMut, TrainReport},
    SetupFnReturn, StationFnReturn, StationId, StationIdInvalidFmt, StationSpecBuilder,
    StationSpecFns,
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
    ///
    /// [`builder`]: Self::builder
    pub fn new(
        id: StationId,
        name: String,
        description: String,
        station_spec_fns: StationSpecFns<E>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            station_spec_fns,
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

    /// Verifies input, calculates progress limit, and inserts resources.
    pub fn setup<'f>(
        &self,
        station: &'f mut StationMut<E>,
        train_report: &'f mut TrainReport<E>,
    ) -> SetupFnReturn<'f, E> {
        let setup_fn = self.station_spec_fns.setup_fn.clone();
        setup_fn.0(station, train_report)
    }

    /// Checks if the station needs to be visited.
    pub fn check<'f>(
        &self,
        station: &'f mut StationMut<E>,
        train_report: &'f TrainReport<E>,
    ) -> Option<StationFnReturn<'f, CheckStatus, E>> {
        self.station_spec_fns
            .check_fn
            .clone()
            .map(move |check_fn| (check_fn.f)(station, train_report))
    }

    /// Returns a task to visit the station.
    pub fn visit<'f>(
        &self,
        station: &'f mut StationMut<E>,
        train_report: &'f TrainReport<E>,
    ) -> StationFnReturn<'f, (), E> {
        let visit_fn = self.station_spec_fns.visit_fn.clone();
        (visit_fn.f)(station, train_report)
    }
}

impl<E> Clone for StationSpec<E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            station_spec_fns: self.station_spec_fns.clone(),
        }
    }
}

impl<E> fmt::Display for StationSpec<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

impl<E> FnMeta for StationSpec<E> {
    fn borrows(&self) -> TypeIds {
        self.station_spec_fns.borrows()
    }

    fn borrow_muts(&self) -> TypeIds {
        self.station_spec_fns.borrow_muts()
    }
}

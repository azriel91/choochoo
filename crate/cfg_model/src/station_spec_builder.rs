use std::convert::TryFrom;

use crate::{CheckStatus, StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns};

/// Builder to make it more ergonomic to construct a [`StationSpec`].
///
/// * If the `name` field is not set, then this will be cloned from the `id`.
/// * If the `description` field is not set, then the empty string will be used.
/// * The `progress_unit` defaults to [`ProgressUnit::None`].
#[derive(Debug)]
pub struct StationSpecBuilder<E> {
    /// Unique identifier of the station.
    id: StationId,
    /// Human readable name of the station.
    name: Option<String>,
    /// Short description of the station's purpose.
    description: Option<String>,
    /// Steps to run when this station is visited.
    station_spec_fns: StationSpecFns<E>,
}

impl<E> StationSpecBuilder<E> {
    /// Returns a new [`StationSpecBuilder`].
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    /// * `station_spec_fns`: Steps to run when this station is visited.
    pub fn new<Id>(
        id: Id,
        station_spec_fns: StationSpecFns<E>,
    ) -> Result<Self, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        let id = StationId::try_from(id)?;
        Ok(StationSpecBuilder {
            id,
            name: None,
            description: None,
            station_spec_fns,
        })
    }

    /// Returns a new [`StationSpecBuilder`] to build a mock [`StationSpec`].
    ///
    /// This defaults the [`StationSpecFns`] to be success / no-op functions.
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    #[cfg(feature = "mock")]
    pub fn mock<Id>(id: Id) -> Result<Self, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        use crate::{ProgressLimit, SetupFn};

        let station_spec_fns = {
            let setup_fn = SetupFn::ok(ProgressLimit::Steps(10));
            let visit_fn = StationFn::ok(());
            StationSpecFns::new(setup_fn, visit_fn)
        };

        Self::new(id, station_spec_fns)
    }

    /// Sets the name of the [`StationSpec`].
    pub fn with_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        self.name = Some(name.into());
        self
    }

    /// Sets the description of the [`StationSpec`].
    pub fn with_description<S>(mut self, description: S) -> Self
    where
        S: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Sets the [`StationSpecFns`] of the [`StationSpec`].
    pub fn with_station_spec_fns(mut self, station_spec_fns: StationSpecFns<E>) -> Self {
        self.station_spec_fns = station_spec_fns;
        self
    }

    /// Sets the check function for the [`StationSpec`].
    pub fn with_check_fn(mut self, check_fn: StationFn<CheckStatus, E>) -> Self {
        self.station_spec_fns.check_fn = Some(check_fn);
        self
    }

    /// Sets the visit function for the [`StationSpec`].
    pub fn with_visit_fn(mut self, visit_fn: StationFn<(), E>) -> Self {
        self.station_spec_fns.visit_fn = visit_fn;
        self
    }

    /// Builds and returns the [`StationSpec`].
    pub fn build(self) -> StationSpec<E> {
        let StationSpecBuilder {
            id,
            name,
            description,
            station_spec_fns,
        } = self;

        let id_ref = &*id;
        let name = name.unwrap_or_else(|| id_ref.clone().into_owned());
        let description = description.unwrap_or_else(String::new);

        StationSpec {
            id,
            name,
            description,
            station_spec_fns,
        }
    }
}

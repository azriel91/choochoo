use std::convert::TryFrom;

use crate::{ProgressUnit, StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns};

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
    /// Unit of measurement to display progress information.
    progress_unit: ProgressUnit,
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
            progress_unit: ProgressUnit::None,
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
        let station_spec_fns = {
            let visit_fn = StationFn::ok(());
            StationSpecFns::new(visit_fn)
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

    /// Sets the progress unit of the [`StationSpec`].
    pub fn with_progress_unit(mut self, progress_unit: ProgressUnit) -> Self {
        self.progress_unit = progress_unit;
        self
    }

    /// Sets the [`StationSpecFns`] of the [`StationSpec`].
    pub fn with_station_spec_fns(mut self, station_spec_fns: StationSpecFns<E>) -> Self {
        self.station_spec_fns = station_spec_fns;
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
            progress_unit,
        } = self;

        let id_ref = &*id;
        let name = name.unwrap_or_else(|| id_ref.clone().into_owned());
        let description = description.unwrap_or_else(String::new);

        StationSpec {
            id,
            name,
            description,
            station_spec_fns,
            progress_unit,
        }
    }
}

use std::convert::TryFrom;

use crate::{
    rt::{CheckStatus, ResIds},
    CreateFns, SetupFn, StationFn, StationId, StationIdInvalidFmt, StationOp, StationSpec,
};

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
    /// Grouping of operations to create and clean up resources.
    station_op: StationOp<E>,
}

impl<E> StationSpecBuilder<E>
where
    E: 'static,
{
    /// Returns a new [`StationSpecBuilder`].
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    /// * `station_op`: Grouping of operations to create and clean up resources.
    pub fn new<Id>(id: Id, station_op: StationOp<E>) -> Result<Self, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        let id = StationId::try_from(id)?;
        Ok(StationSpecBuilder {
            id,
            name: None,
            description: None,
            station_op,
        })
    }

    /// Returns a new [`StationSpecBuilder`] to build a mock [`StationSpec`].
    ///
    /// This defaults the [`CreateFns`] to be success / no-op functions.
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    #[cfg(feature = "mock")]
    pub fn mock<Id>(id: Id) -> Result<Self, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        use crate::rt::ProgressLimit;

        let station_op = {
            let setup_fn = SetupFn::<E>::ok(ProgressLimit::Steps(10));
            let work_fn = StationFn::ok(ResIds::new());
            let create_fns = CreateFns::new(setup_fn, work_fn);
            let clean_op_fns = None;

            StationOp::new(create_fns, clean_op_fns)
        };

        Self::new(id, station_op)
    }

    /// Sets the name of the [`StationSpec`].
    #[must_use]
    pub fn with_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        self.name = Some(name.into());
        self
    }

    /// Sets the description of the [`StationSpec`].
    #[must_use]
    pub fn with_description<S>(mut self, description: S) -> Self
    where
        S: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Sets the [`CreateFns`] of the [`StationSpec`].
    #[must_use]
    pub fn with_station_op(mut self, station_op: StationOp<E>) -> Self {
        self.station_op = station_op;
        self
    }

    /// Sets the check function for the [`StationSpec`].
    #[must_use]
    pub fn with_setup_fn(mut self, setup_fn: SetupFn<E>) -> Self {
        self.station_op.create_fns.setup_fn = setup_fn;
        self
    }

    /// Sets the check function for the [`StationSpec`].
    #[must_use]
    pub fn with_check_fn(mut self, check_fn: StationFn<CheckStatus, E, E>) -> Self {
        self.station_op.create_fns.check_fn = Some(check_fn);
        self
    }

    /// Sets the visit function for the [`StationSpec`].
    #[must_use]
    pub fn with_work_fn(mut self, work_fn: StationFn<ResIds, (ResIds, E), E>) -> Self {
        self.station_op.create_fns.work_fn = work_fn;
        self
    }

    /// Builds and returns the [`StationSpec`].
    pub fn build(self) -> StationSpec<E> {
        let StationSpecBuilder {
            id,
            name,
            description,
            station_op,
        } = self;

        let id_ref = &*id;
        let name = name.unwrap_or_else(|| id_ref.clone().into_owned());
        let description = description.unwrap_or_default();

        StationSpec {
            id,
            name,
            description,
            station_op,
        }
    }
}

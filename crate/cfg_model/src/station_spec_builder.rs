use std::convert::TryFrom;

use crate::{
    rt::CheckStatus, OpFns, SetupFn, StationFn, StationId, StationIdInvalidFmt, StationSpec,
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
    /// Steps to run when this station is visited.
    op_fns: OpFns<E>,
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
    /// * `op_fns`: Steps to run when this station is visited.
    pub fn new<Id>(id: Id, op_fns: OpFns<E>) -> Result<Self, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        let id = StationId::try_from(id)?;
        Ok(StationSpecBuilder {
            id,
            name: None,
            description: None,
            op_fns,
        })
    }

    /// Returns a new [`StationSpecBuilder`] to build a mock [`StationSpec`].
    ///
    /// This defaults the [`OpFns`] to be success / no-op functions.
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

        let op_fns = {
            let setup_fn = SetupFn::ok(ProgressLimit::Steps(10));
            let visit_fn = StationFn::ok(());
            OpFns::new(setup_fn, visit_fn)
        };

        Self::new(id, op_fns)
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

    /// Sets the [`OpFns`] of the [`StationSpec`].
    #[must_use]
    pub fn with_op_fns(mut self, op_fns: OpFns<E>) -> Self {
        self.op_fns = op_fns;
        self
    }

    /// Sets the check function for the [`StationSpec`].
    #[must_use]
    pub fn with_setup_fn(mut self, setup_fn: SetupFn<E>) -> Self {
        self.op_fns.setup_fn = setup_fn;
        self
    }

    /// Sets the check function for the [`StationSpec`].
    #[must_use]
    pub fn with_check_fn(mut self, check_fn: StationFn<CheckStatus, E>) -> Self {
        self.op_fns.check_fn = Some(check_fn);
        self
    }

    /// Sets the visit function for the [`StationSpec`].
    #[must_use]
    pub fn with_visit_fn(mut self, visit_fn: StationFn<(), E>) -> Self {
        self.op_fns.visit_fn = visit_fn;
        self
    }

    /// Builds and returns the [`StationSpec`].
    pub fn build(self) -> StationSpec<E> {
        let StationSpecBuilder {
            id,
            name,
            description,
            op_fns,
        } = self;

        let id_ref = &*id;
        let name = name.unwrap_or_else(|| id_ref.clone().into_owned());
        let description = description.unwrap_or_default();

        StationSpec {
            id,
            name,
            description,
            op_fns,
        }
    }
}

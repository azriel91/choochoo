use std::{convert::TryFrom, fmt};

use fn_graph::{FnMeta, TypeIds};

use crate::{OpFns, StationId, StationIdInvalidFmt, StationSpecBuilder};

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
    pub(crate) op_fns: OpFns<E>,
}

impl<E> StationSpec<E>
where
    E: 'static,
{
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
    /// * `op_fns`: Steps to run when this station is visited.
    ///
    /// [`builder`]: Self::builder
    pub fn new(id: StationId, name: String, description: String, op_fns: OpFns<E>) -> Self {
        Self {
            id,
            name,
            description,
            op_fns,
        }
    }

    /// Returns a new [`StationSpecBuilder`].
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    /// * `op_fns`: Steps to run when this station is visited.
    pub fn builder<Id>(
        id: Id,
        op_fns: OpFns<E>,
    ) -> Result<StationSpecBuilder<E>, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        StationSpecBuilder::new(id, op_fns)
    }

    /// Returns a new [`StationSpecBuilder`] to build a mock [`StationSpec`].
    ///
    /// This defaults the [`OpFns`] to be success / no-op functions.
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
    pub fn op_fns(&self) -> &OpFns<E> {
        &self.op_fns
    }
}

impl<E> Clone for StationSpec<E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            op_fns: self.op_fns.clone(),
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
        self.op_fns.borrows()
    }

    fn borrow_muts(&self) -> TypeIds {
        self.op_fns.borrow_muts()
    }
}

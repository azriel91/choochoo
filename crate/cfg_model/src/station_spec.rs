use std::{convert::TryFrom, fmt};

use fn_graph::{FnMeta, TypeIds};

use crate::{StationId, StationIdInvalidFmt, StationOp, StationSpecBuilder};

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
    /// Grouping of operations to create and clean up resources.
    pub(crate) station_op: StationOp<E>,
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
    /// * `station_op`: Grouping of operations to create and clean up resources.
    ///
    /// [`builder`]: Self::builder
    pub fn new(id: StationId, name: String, description: String, station_op: StationOp<E>) -> Self {
        Self {
            id,
            name,
            description,
            station_op,
        }
    }

    /// Returns a new [`StationSpecBuilder`].
    ///
    /// # Parameters
    ///
    /// * `id`: Unique identifier of the station.
    /// * `station_op`: Grouping of operations to create and clean up resources.
    pub fn builder<Id>(
        id: Id,
        station_op: StationOp<E>,
    ) -> Result<StationSpecBuilder<E>, StationIdInvalidFmt<'static>>
    where
        StationId: TryFrom<Id, Error = StationIdInvalidFmt<'static>>,
    {
        StationSpecBuilder::new(id, station_op)
    }

    /// Returns a new [`StationSpecBuilder`] to build a mock [`StationSpec`].
    ///
    /// This defaults the [`StationOp`] to be success / no-op functions.
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
    pub fn station_op(&self) -> &StationOp<E> {
        &self.station_op
    }
}

impl<E> Clone for StationSpec<E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            station_op: self.station_op.clone(),
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
        self.station_op.create_fns().borrows()
    }

    fn borrow_muts(&self) -> TypeIds {
        self.station_op.create_fns().borrow_muts()
    }
}

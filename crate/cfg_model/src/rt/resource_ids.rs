use std::ops::{Deref, DerefMut};

use type_reg::untagged::TypeMap;

use crate::rt::ResourceIdLogical;

/// List of [`ResourceIdPhysical`]s, `TypeMap<ResourceIdLogical>` newtype.
///
/// This should be `Deserialize, Serialize`, but TypeId is not const, and may
/// not ever be across compiler versions.
///
/// TypeMap<String, ..> for users to keep track of their resources as strings?
/// Maybe, but we want them to get back a strong type, from which they can
/// reason how to delete it.
///
/// `RtMap` does not require `Resource` types to impl Deserialize + Serialize,
/// which is sensible for runtime values. We would have to create a new trait
/// and a new map type if we wanted that. However the TypeId key serialization
/// problem is still there.
#[derive(Clone, Debug, Default)]
pub struct ResourceIds(pub TypeMap<ResourceIdLogical>);

impl ResourceIds {
    /// Returns an empty map of resource IDs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an empty map of resource IDs with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(TypeMap::with_capacity(capacity))
    }
}

impl Deref for ResourceIds {
    type Target = TypeMap<ResourceIdLogical>;

    #[cfg(not(tarpaulin_include))]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ResourceIds {
    #[cfg(not(tarpaulin_include))]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

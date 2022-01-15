use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::rt::{ResourceIdLogical, ResourceIdPhysical};

/// List of [`ResourceIdPhysical`]s, `HashMap<ResourceIdLogical,
/// ResourceIdPhysical>` newtype.
///
/// This should be `Deserialize, Serialize`, but TypeId is not const, and may
/// not ever be across compiler versions.
///
/// HashMap<String, ..> for users to keep track of their resources as strings?
/// Maybe, but we want them to get back a strong type, from which they can
/// reason how to delete it.
///
/// `RtMap` does not require `Resource` types to impl Deserialize + Serialize,
/// which is sensible for runtime values. We would have to create a new trait
/// and a new map type if we wanted that. However the TypeId key serialization
/// problem is still there.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ResourceIds(pub HashMap<ResourceIdLogical, ResourceIdPhysical>);

impl ResourceIds {
    /// Returns an empty map of resource IDs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an empty map of resource IDs with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }
}

impl Deref for ResourceIds {
    type Target = HashMap<ResourceIdLogical, ResourceIdPhysical>;

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

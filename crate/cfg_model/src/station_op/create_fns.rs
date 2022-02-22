use crate::{rt::ResourceIds, OpFns};

/// Functions for creating an operation's resources.
pub type CreateFns<E> = OpFns<ResourceIds, (ResourceIds, E), E>;

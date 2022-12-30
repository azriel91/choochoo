use crate::{rt::ResIds, OpFns};

/// Functions for creating an operation's resources.
pub type CreateFns<E> = OpFns<ResIds, (ResIds, E), E>;

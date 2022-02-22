use crate::OpFns;

/// Functions for cleaning an operation's resources.
pub type CleanFns<E> = OpFns<(), E, E>;

use crate::{rt::ResourceIds, OpFns};

// **Note:** `Clone` is manually implemented to avoid the trait bound on `E`.
/// Grouping of operations to create and clean up resources.
#[derive(Debug)]
pub struct StationOp<E> {
    /// Steps to run when this station is visited.
    pub(crate) create_op_fns: OpFns<ResourceIds, E>,
    /// Steps to run to clean up the station.
    pub(crate) clean_op_fns: Option<OpFns<(), E>>,
}

impl<E> StationOp<E> {
    /// Returns a new `StationOp`.
    pub fn new(create_op_fns: OpFns<ResourceIds, E>, clean_op_fns: Option<OpFns<(), E>>) -> Self {
        Self {
            create_op_fns,
            clean_op_fns,
        }
    }

    /// Returns this station's [`OpFns`] for creating resources.
    pub fn create_op_fns(&self) -> &OpFns<ResourceIds, E> {
        &self.create_op_fns
    }

    /// Returns this station's [`OpFns`] for cleaning up resources.
    pub fn clean_op_fns(&self) -> Option<&OpFns<(), E>> {
        self.clean_op_fns.as_ref()
    }
}

impl<E> Clone for StationOp<E> {
    fn clone(&self) -> Self {
        Self {
            create_op_fns: self.create_op_fns.clone(),
            clean_op_fns: self.clean_op_fns.clone(),
        }
    }
}

impl<E> PartialEq for StationOp<E> {
    fn eq(&self, other: &Self) -> bool {
        self.create_op_fns.eq(&other.create_op_fns) && self.clean_op_fns.eq(&other.clean_op_fns)
    }
}

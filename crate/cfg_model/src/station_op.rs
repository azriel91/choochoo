pub use self::{clean_fns::CleanFns, create_fns::CreateFns, op_fns::OpFns};

mod clean_fns;
mod create_fns;
mod op_fns;

// **Note:** `Clone` is manually implemented to avoid the trait bound on `E`.
/// Grouping of operations to create and clean up resources.
#[derive(Debug)]
pub struct StationOp<E> {
    /// Steps to run when this station is visited.
    pub(crate) create_fns: CreateFns<E>,
    /// Steps to run to clean up the station.
    pub(crate) clean_fns: Option<CleanFns<E>>,
}

impl<E> StationOp<E> {
    /// Returns a new `StationOp`.
    pub fn new(create_fns: CreateFns<E>, clean_fns: Option<CleanFns<E>>) -> Self {
        Self {
            create_fns,
            clean_fns,
        }
    }

    /// Returns this station's [`OpFns`] for creating resources.
    pub fn create_fns(&self) -> &CreateFns<E> {
        &self.create_fns
    }

    /// Returns this station's [`OpFns`] for cleaning up resources.
    pub fn clean_fns(&self) -> Option<&CleanFns<E>> {
        self.clean_fns.as_ref()
    }
}

impl<E> Clone for StationOp<E> {
    fn clone(&self) -> Self {
        Self {
            create_fns: self.create_fns.clone(),
            clean_fns: self.clean_fns.clone(),
        }
    }
}

impl<E> PartialEq for StationOp<E> {
    fn eq(&self, other: &Self) -> bool {
        self.create_fns.eq(&other.create_fns) && self.clean_fns.eq(&other.clean_fns)
    }
}

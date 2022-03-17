use std::fmt;

/// Operation to run when visiting stations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitOp {
    /// Create the resources for this station.
    Create,
    /// Clean up the resources produced at this station.
    Clean,
}

impl fmt::Display for VisitOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create => "create".fmt(f),
            Self::Clean => "clean".fmt(f),
        }
    }
}

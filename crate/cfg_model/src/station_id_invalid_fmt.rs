use std::{borrow::Cow, fmt};

/// Error indicating station ID provided is not in the correct format.
#[derive(Debug, PartialEq, Eq)]
pub struct StationIdInvalidFmt<'s> {
    /// String that was provided for the station ID.
    value: Cow<'s, str>,
}

impl<'s> StationIdInvalidFmt<'s> {
    /// Returns a new `StationIdInvalidFmt`.
    pub fn new(value: Cow<'s, str>) -> Self {
        Self { value }
    }

    /// Returns the value that failed to be parsed as a [`StationId`].
    ///
    /// [`StationId`]: crate::StationId
    pub fn value(&self) -> &Cow<'s, str> {
        &self.value
    }
}

impl<'s> fmt::Display for StationIdInvalidFmt<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` is not a valid station ID. Station IDs can only contain letters, numbers, and underscores.",
            self.value
        )
    }
}

impl<'s> std::error::Error for StationIdInvalidFmt<'s> {}

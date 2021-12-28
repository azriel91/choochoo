use std::fmt;

/// Error representing a string that is not a valid profile name.
///
/// Profiles must be non-empty, and all characters must be lowercase,
/// alphanumeric or underscore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileError(pub String);

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ProfileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

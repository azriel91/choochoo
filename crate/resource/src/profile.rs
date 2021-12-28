use std::ops::Deref;

use crate::ProfileError;

/// Execution profile identifier.
///
/// This is the top level namespace that should logically distinguish different
/// invocations / executions of the tasks.
///
/// Profiles must be non-empty, and all characters must be lowercase,
/// alphanumeric or underscore.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Profile(String);

impl Profile {
    /// Name of the default profile.
    pub const DEFAULT_STR: &'static str = "default";

    /// Returns a new [`Profile`].
    pub fn new<S>(s: S) -> Result<Self, ProfileError>
    where
        S: Into<String>,
    {
        let s = Into::<String>::into(s);
        if s.is_empty() {
            return Err(ProfileError(s));
        }

        if s.chars()
            .all(Self::is_ascii_lowercase_alphanumeric_underscore)
        {
            Ok(Self(s))
        } else {
            Err(ProfileError(s))
        }
    }

    fn is_ascii_lowercase_alphanumeric_underscore(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_')
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self(String::from(Profile::DEFAULT_STR))
    }
}

impl Deref for Profile {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Profile {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

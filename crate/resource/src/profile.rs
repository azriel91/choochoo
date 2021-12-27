use std::ops::Deref;

const PROFILE_DEFAULT_STR: &'static str = "default";

/// Execution profile identifier.
///
/// This is the top level namespace that should logically distinguish different
/// invocations / executions of the tasks.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Profile(String);

impl Profile {
    /// Returns a new [`Profile`].
    pub fn new(profile: String) -> Self {
        Self(profile)
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self(String::from(PROFILE_DEFAULT_STR))
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

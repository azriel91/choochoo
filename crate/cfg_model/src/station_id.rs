use std::{
    borrow::Cow,
    convert::TryFrom,
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::StationIdInvalidFmt;

/// Unique identifier for a Station, `Cow<'static, str>` newtype.
///
/// Can only contain ASCII letters, numbers, and underscores.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StationId(Cow<'static, str>);

impl StationId {
    /// Returns a `StationId` if the given `&str` is valid.
    pub fn new(s: &'static str) -> Result<Self, StationIdInvalidFmt> {
        Self::try_from(s)
    }

    /// Returns whether the provided `&str` is a valid station identifier.
    pub fn is_valid_id(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

impl Deref for StationId {
    type Target = Cow<'static, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StationId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for StationId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for StationId {
    type Error = StationIdInvalidFmt<'static>;

    fn try_from(s: String) -> Result<StationId, StationIdInvalidFmt<'static>> {
        if Self::is_valid_id(&s) {
            Ok(StationId(Cow::Owned(s)))
        } else {
            let s = Cow::Owned(s);
            Err(StationIdInvalidFmt::new(s))
        }
    }
}

impl TryFrom<&'static str> for StationId {
    type Error = StationIdInvalidFmt<'static>;

    fn try_from(s: &'static str) -> Result<StationId, StationIdInvalidFmt<'static>> {
        if Self::is_valid_id(s) {
            Ok(StationId(Cow::Borrowed(s)))
        } else {
            let s = Cow::Borrowed(s);
            Err(StationIdInvalidFmt::new(s))
        }
    }
}

impl FromStr for StationId {
    type Err = StationIdInvalidFmt<'static>;

    fn from_str(s: &str) -> Result<StationId, StationIdInvalidFmt<'static>> {
        if Self::is_valid_id(s) {
            Ok(StationId(Cow::Owned(String::from(s))))
        } else {
            let s = Cow::Owned(String::from(s));
            Err(StationIdInvalidFmt::new(s))
        }
    }
}

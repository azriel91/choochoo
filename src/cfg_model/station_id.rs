use std::{
    borrow::Cow,
    convert::TryFrom,
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::cfg_model::StationIdInvalidFmt;

/// Unique identifier for a Station, `Cow<'static, str>` newtype.
///
/// Can only contain ASCII letters, numbers, and underscores.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, convert::TryFrom, str::FromStr};

    use super::StationId;
    use crate::cfg_model::StationIdInvalidFmt;

    #[test]
    fn from_str_returns_ok_owned_for_valid_id() -> Result<(), StationIdInvalidFmt<'static>> {
        let station_id = StationId::from_str("good_id")?;

        assert_eq!("good_id", *station_id);
        Ok(())
    }

    #[test]
    fn try_from_str_returns_ok_borrowed_for_valid_id() -> Result<(), StationIdInvalidFmt<'static>> {
        let station_id = StationId::try_from("good_id")?;

        assert_eq!("good_id", *station_id);
        Ok(())
    }

    #[test]
    fn try_from_string_returns_ok_owned_for_valid_id() -> Result<(), StationIdInvalidFmt<'static>> {
        let station_id = StationId::try_from(String::from("good_id"))?;

        assert_eq!("good_id", *station_id);
        Ok(())
    }

    #[test]
    fn from_str_returns_err_owned_for_invalid_id() {
        let result = StationId::from_str("has space");

        // Note: We cannot test for ownership until https://github.com/rust-lang/rust/issues/65143 is implemented.
        assert_eq!(
            Err(StationIdInvalidFmt::new(Cow::Owned(String::from(
                "has space"
            )))),
            result
        );
    }

    #[test]
    fn try_from_str_returns_err_borrowed_for_invalid_id() {
        let result = StationId::try_from("has space");

        // Note: We cannot test for ownership until https://github.com/rust-lang/rust/issues/65143 is implemented.
        assert_eq!(
            Err(StationIdInvalidFmt::new(Cow::Borrowed("has space"))),
            result
        );
    }

    #[test]
    fn try_from_string_returns_err_owned_for_invalid_id() {
        let result = StationId::try_from(String::from("has space"));

        // Note: We cannot test for ownership until https://github.com/rust-lang/rust/issues/65143 is implemented.
        assert_eq!(
            Err(StationIdInvalidFmt::new(Cow::Owned(String::from(
                "has space"
            )))),
            result
        );
    }
}

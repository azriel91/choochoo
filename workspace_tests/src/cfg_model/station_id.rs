use std::{borrow::Cow, convert::TryFrom, str::FromStr};

use choochoo_cfg_model::{StationId, StationIdInvalidFmt};

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

#[test]
fn display_returns_inner_str() -> Result<(), StationIdInvalidFmt<'static>> {
    let station_id = StationId::try_from("good_id")?;

    assert_eq!("good_id", station_id.to_string());
    Ok(())
}

use std::borrow::Cow;

use choochoo_cfg_model::StationIdInvalidFmt;

#[test]
fn display_returns_readable_message() {
    let station_id_invalid_fmt = StationIdInvalidFmt::new(Cow::Borrowed("a b c"));

    assert_eq!(
        "`a b c` is not a valid station ID. Station IDs can only contain letters, numbers, and underscores.",
        station_id_invalid_fmt.to_string()
    );
}

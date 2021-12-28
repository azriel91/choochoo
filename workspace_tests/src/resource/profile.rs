use choochoo_resource::{Profile, ProfileError};

#[test]
fn empty_string_returns_error() {
    assert_eq!(Err(ProfileError(String::from(""))), Profile::new(""));
}

#[test]
fn whitespace_in_string_returns_error() {
    assert_eq!(Err(ProfileError(String::from("a b"))), Profile::new("a b"));
}

#[test]
fn uppercase_letter_returns_error() {
    assert_eq!(Err(ProfileError(String::from("A"))), Profile::new("A"));
}

#[test]
fn underscore_is_valid() {
    assert!(Profile::new("__").is_ok());
}

#[test]
fn numbers_are_valid() {
    assert!(Profile::new("0123456789").is_ok());
}

#[test]
fn lowercase_letters_are_valid() {
    assert!(Profile::new("abcdefghijklmnopqrstuvwxyz").is_ok());
}

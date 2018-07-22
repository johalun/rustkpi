extern crate utf8;

#[path = "shared/data.rs"]
mod data;

#[path = "shared/string_from_utf8_lossy.rs"]
mod string_from_utf8_lossy;

#[test]
fn test_string_from_utf8_lossy() {
    for &(input, expected) in data::DECODED_LOSSY {
        assert_eq!(string_from_utf8_lossy::string_from_utf8_lossy(input), expected);
    }
}

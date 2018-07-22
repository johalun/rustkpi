extern crate utf8;

/// A re-implementation of std::str::from_utf8
pub fn str_from_utf8(input: &[u8]) -> Result<&str, usize> {
    match utf8::decode(input) {
        Ok(s) => return Ok(s),
        Err(utf8::DecodeError::Invalid { valid_prefix, .. }) |
        Err(utf8::DecodeError::Incomplete { valid_prefix, .. }) => Err(valid_prefix.len()),
    }
}

#[test]
fn test_str_from_utf8() {
    let xs = b"hello";
    assert_eq!(str_from_utf8(xs), Ok("hello"));

    let xs = "ศไทย中华Việt Nam".as_bytes();
    assert_eq!(str_from_utf8(xs), Ok("ศไทย中华Việt Nam"));

    let xs = b"hello\xFF";
    assert!(str_from_utf8(xs).is_err());
}

#[test]
fn test_is_utf8() {
    // Chars of 1, 2, 3, and 4 bytes
    assert!(str_from_utf8("eé€\u{10000}".as_bytes()).is_ok());
    // invalid prefix
    assert!(str_from_utf8(&[0x80]).is_err());
    // invalid 2 byte prefix
    assert!(str_from_utf8(&[0xc0]).is_err());
    assert!(str_from_utf8(&[0xc0, 0x10]).is_err());
    // invalid 3 byte prefix
    assert!(str_from_utf8(&[0xe0]).is_err());
    assert!(str_from_utf8(&[0xe0, 0x10]).is_err());
    assert!(str_from_utf8(&[0xe0, 0xff, 0x10]).is_err());
    // invalid 4 byte prefix
    assert!(str_from_utf8(&[0xf0]).is_err());
    assert!(str_from_utf8(&[0xf0, 0x10]).is_err());
    assert!(str_from_utf8(&[0xf0, 0xff, 0x10]).is_err());
    assert!(str_from_utf8(&[0xf0, 0xff, 0xff, 0x10]).is_err());

    // deny overlong encodings
    assert!(str_from_utf8(&[0xc0, 0x80]).is_err());
    assert!(str_from_utf8(&[0xc0, 0xae]).is_err());
    assert!(str_from_utf8(&[0xe0, 0x80, 0x80]).is_err());
    assert!(str_from_utf8(&[0xe0, 0x80, 0xaf]).is_err());
    assert!(str_from_utf8(&[0xe0, 0x81, 0x81]).is_err());
    assert!(str_from_utf8(&[0xf0, 0x82, 0x82, 0xac]).is_err());
    assert!(str_from_utf8(&[0xf4, 0x90, 0x80, 0x80]).is_err());

    // deny surrogates
    assert!(str_from_utf8(&[0xED, 0xA0, 0x80]).is_err());
    assert!(str_from_utf8(&[0xED, 0xBF, 0xBF]).is_err());

    assert!(str_from_utf8(&[0xC2, 0x80]).is_ok());
    assert!(str_from_utf8(&[0xDF, 0xBF]).is_ok());
    assert!(str_from_utf8(&[0xE0, 0xA0, 0x80]).is_ok());
    assert!(str_from_utf8(&[0xED, 0x9F, 0xBF]).is_ok());
    assert!(str_from_utf8(&[0xEE, 0x80, 0x80]).is_ok());
    assert!(str_from_utf8(&[0xEF, 0xBF, 0xBF]).is_ok());
    assert!(str_from_utf8(&[0xF0, 0x90, 0x80, 0x80]).is_ok());
    assert!(str_from_utf8(&[0xF4, 0x8F, 0xBF, 0xBF]).is_ok());
}

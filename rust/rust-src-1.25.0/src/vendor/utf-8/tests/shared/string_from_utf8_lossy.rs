use std::borrow::Cow;
use utf8::{decode, DecodeError, REPLACEMENT_CHARACTER};

/// A re-implementation of String::from_utf8_lossy
pub fn string_from_utf8_lossy(input: &[u8]) -> Cow<str> {
    let mut result = decode(input);
    if let Ok(s) = result {
        return s.into()
    }
    let mut string = String::with_capacity(input.len() + REPLACEMENT_CHARACTER.len());
    loop {
        match result {
            Ok(s) => {
                string.push_str(s);
                return string.into()
            }
            Err(DecodeError::Incomplete { valid_prefix, .. }) => {
                string.push_str(valid_prefix);
                string.push_str(REPLACEMENT_CHARACTER);
                return string.into()
            }
            Err(DecodeError::Invalid { valid_prefix, remaining_input, .. }) => {
                string.push_str(valid_prefix);
                string.push_str(REPLACEMENT_CHARACTER);
                result = decode(remaining_input);
            }
        }
    }
}

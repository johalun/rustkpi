#[macro_use] extern crate matches;

mod polyfill;
mod lossy;

pub use lossy::LossyDecoder;

use std::cmp;
use std::str;

/// The replacement character, U+FFFD. In lossy decoding, insert it for every decoding error.
pub const REPLACEMENT_CHARACTER: &'static str = "\u{FFFD}";

#[derive(Debug, Copy, Clone)]
pub enum DecodeError<'a> {
    /// In lossy decoding insert `valid_prefix`, then `"\u{FFFD}"`,
    /// then call `decode()` again with `remaining_input`.
    Invalid {
        valid_prefix: &'a str,
        invalid_sequence: &'a [u8],
        remaining_input: &'a [u8],
    },

    /// Call the `incomplete_suffix.try_complete` method with more input when available.
    /// If no more input is available, this is an invalid byte sequence.
    Incomplete {
        valid_prefix: &'a str,
        incomplete_suffix: Incomplete,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct Incomplete {
    pub buffer: [u8; 4],
    pub buffer_len: u8,
}

pub fn decode(input: &[u8]) -> Result<&str, DecodeError> {
    let error = match str::from_utf8(input) {
        Ok(valid) => return Ok(valid),
        Err(error) => error,
    };

    // FIXME: separate function from here to guide inlining?
    let (valid, after_valid) = input.split_at(error.valid_up_to());
    let valid = unsafe {
        str::from_utf8_unchecked(valid)
    };

    match polyfill::utf8error_error_len(&error, input) {
        Some(invalid_sequence_length) => {
            let (invalid, rest) = after_valid.split_at(invalid_sequence_length);
            Err(DecodeError::Invalid {
                valid_prefix: valid,
                invalid_sequence: invalid,
                remaining_input: rest
            })
        }
        None => {
            Err(DecodeError::Incomplete {
                valid_prefix: valid,
                incomplete_suffix: Incomplete::new(after_valid),
            })
        }
    }
}

impl Incomplete {
    pub fn empty() -> Self {
        Incomplete {
            buffer: [0, 0, 0, 0],
            buffer_len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer_len == 0
    }

    pub fn new(bytes: &[u8]) -> Self {
        let mut buffer = [0, 0, 0, 0];
        let len = bytes.len();
        buffer[..len].copy_from_slice(bytes);
        Incomplete {
            buffer: buffer,
            buffer_len: len as u8,
        }
    }

    /// * `None`: still incomplete, call `try_complete` again with more input.
    ///   If no more input is available, this is invalid byte sequence.
    /// * `Some((result, remaining_input))`: We’re done with this `Incomplete`.
    ///   To keep decoding, pass `remaining_input` to `decode()`.
    pub fn try_complete<'input>(&mut self, input: &'input [u8])
                                -> Option<(Result<&str, &[u8]>, &'input [u8])> {
        let buffer_len = self.buffer_len as usize;
        let copied_from_input;
        {
            let unwritten = &mut self.buffer[buffer_len..];
            copied_from_input = cmp::min(unwritten.len(), input.len());
            unwritten[..copied_from_input].copy_from_slice(&input[..copied_from_input]);
        }
        let spliced = &self.buffer[..buffer_len + copied_from_input];
        match str::from_utf8(spliced) {
            Ok(valid) => {
                self.buffer_len = 0;
                Some((Ok(valid), &input[copied_from_input..]))
            }
            Err(error) => {
                let valid_up_to = error.valid_up_to();
                if valid_up_to > 0 {
                    let valid = &self.buffer[..valid_up_to];
                    let valid = unsafe {
                        str::from_utf8_unchecked(valid)
                    };
                    let consumed = valid_up_to.checked_sub(buffer_len).unwrap();
                    self.buffer_len = 0;
                    Some((Ok(valid), &input[consumed..]))
                } else {
                    match polyfill::utf8error_error_len(&error, spliced) {
                        Some(invalid_sequence_length) => {
                            let invalid = &spliced[..invalid_sequence_length];
                            let consumed = invalid_sequence_length.checked_sub(buffer_len).unwrap();
                            let rest = &input[consumed..];
                            self.buffer_len = 0;
                            Some((Err(invalid), rest))
                        }
                        None => {
                            self.buffer_len = spliced.len() as u8;
                            None
                        }
                    }
                }
            }
        }
    }
}

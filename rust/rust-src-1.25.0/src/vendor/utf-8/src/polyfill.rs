use std::str::Utf8Error;

/// Replace with `error.error_len()` when https://github.com/rust-lang/rust/issues/40494 is stable
pub fn utf8error_error_len(error: &Utf8Error, input: &[u8]) -> Option<usize> {
    let after_valid = &input[error.valid_up_to()..];

    // `after_valid` is not empty, `str::from_utf8` would have returned `Ok(_)`.
    let first = after_valid[0];
    let char_width = UTF8_CHAR_WIDTH[first as usize];

    macro_rules! get_byte {
        ($i: expr) => {
            if let Some(&byte) = after_valid.get($i) {
                byte
            } else {
                return None
            }
        }
    }

    let invalid_sequence_length;
    match char_width {
        0 => invalid_sequence_length = 1,
        1 => panic!("found ASCII byte after Utf8Error.valid_up_to()"),
        2 => {
            let second = get_byte!(1);
            debug_assert!(!is_continuation_byte(second));
            invalid_sequence_length = 1;
        }
        3 => {
            let second = get_byte!(1);
            if valid_three_bytes_sequence_prefix(first, second) {
                let third = get_byte!(2);
                debug_assert!(!is_continuation_byte(third));
                invalid_sequence_length = 2;
            } else {
                invalid_sequence_length = 1;
            }
        }
        4 => {
            let second = get_byte!(1);
            if valid_four_bytes_sequence_prefix(first, second) {
                let third = get_byte!(2);
                if is_continuation_byte(third) {
                    let fourth = get_byte!(3);
                    debug_assert!(!is_continuation_byte(fourth));
                    invalid_sequence_length = 3;
                } else {
                    invalid_sequence_length = 2;
                }
            } else {
                invalid_sequence_length = 1;
            }
        }
        _ => unreachable!()
    }

    Some(invalid_sequence_length)
}

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
    0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
    2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
    3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
    4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

#[inline]
fn is_continuation_byte(b: u8) -> bool {
    const CONTINUATION_MASK: u8 = 0b1100_0000;
    const CONTINUATION_TAG: u8 = 0b1000_0000;
    b & CONTINUATION_MASK == CONTINUATION_TAG
}

#[inline]
fn valid_three_bytes_sequence_prefix(first: u8, second: u8) -> bool {
    matches!((first, second),
        (0xE0         , 0xA0 ... 0xBF) |
        (0xE1 ... 0xEC, 0x80 ... 0xBF) |
        (0xED         , 0x80 ... 0x9F) |
        // Exclude surrogates: (0xED, 0xA0 ... 0xBF)
        (0xEE ... 0xEF, 0x80 ... 0xBF)
    )
}

#[inline]
fn valid_four_bytes_sequence_prefix(first: u8, second: u8) -> bool {
    matches!((first, second),
        (0xF0         , 0x90 ... 0xBF) |
        (0xF1 ... 0xF3, 0x80 ... 0xBF) |
        (0xF4         , 0x80 ... 0x8F)
    )
}

pub const DECODED_LOSSY: &'static [(&'static [u8], &'static str)] = &[
    (b"hello", "hello"),
    (b"\xe0\xb8\xa8\xe0\xb9\x84\xe0\xb8\x97\xe0\xb8\xa2\xe4\xb8\xad\xe5\x8d\x8e", "ศไทย中华"),
    (b"Vi\xe1\xbb\x87t Nam", "Việt Nam"),
    (b"Hello\xC2 There\xFF ", "Hello\u{FFFD} There\u{FFFD} "),
    (b"Hello\xC0\x80 There", "Hello\u{FFFD}\u{FFFD} There"),
    (b"\xE6\x83 Goodbye", "\u{FFFD} Goodbye"),
    (b"\xF5foo\xF5\x80bar", "\u{FFFD}foo\u{FFFD}\u{FFFD}bar"),
    (b"\xF5foo\xF5\xC2", "\u{FFFD}foo\u{FFFD}\u{FFFD}"),
    (b"\xF1foo\xF1\x80bar\xF1\x80\x80baz", "\u{FFFD}foo\u{FFFD}bar\u{FFFD}baz"),
    (b"\xF4foo\xF4\x80bar\xF4\xBFbaz", "\u{FFFD}foo\u{FFFD}bar\u{FFFD}\u{FFFD}baz"),
    (b"\xF0\x80\x80\x80foo\xF0\x90\x80\x80bar", "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}foo\u{10000}bar"),
    (b"\xF0\x90\x80foo", "\u{FFFD}foo"),
    // surrogates
    (b"\xED\xA0\x80foo\xED\xBF\xBFbar", "\u{FFFD}\u{FFFD}\u{FFFD}foo\u{FFFD}\u{FFFD}\u{FFFD}bar"),
];

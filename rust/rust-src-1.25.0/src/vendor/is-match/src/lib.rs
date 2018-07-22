#[macro_export]
macro_rules! is_match {
    ($expression: expr, $($pattern:tt)+) => {
        is_match! {tt
            match $expression {
                $($pattern)+ => true,
                _            => false
            }
        }
    };
    (tt $value:expr) => ($value);
}

#[test]
fn test_matching() {
    let foo = Some("-12");
    assert!(is_match!(foo, Some(bar) if
        is_match!(bar.as_bytes()[0], b'+' | b'-') &&
        is_match!(bar.as_bytes()[1], b'0'...b'9')
    ));
    assert!(!is_match!(foo, None));
}


extern crate log_settings;

#[test]
fn smoke() {
    log_settings::settings().indentation = 5;
}

#[test]
fn set_get() {
    {
        log_settings::settings().indentation = 6;
    }
    assert_eq!(log_settings::settings().indentation, 6);
}

#[test]
fn set_get2() {
    log_settings::settings().indentation = 42;
    assert_eq!(log_settings::settings().indentation, 42);
}

extern crate log_surgeon;

#[test]
fn test_log_surgeon_version() {
    assert_eq!(log_surgeon::version(), "0.0.1");
}

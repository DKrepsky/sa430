
#[test]
#[cfg(target_os = "linux")]
fn given_target_is_linux_when_create_then_create_a_linux_scanner() {
    let result = ScannerFactory::create();
    assert!(result.is_ok());
}

#[test]
#[cfg(not(target_os = "linux"))]
fn given_target_is_unknown_when_create_then_panic() {
    assert!(ScannerFactory::create().is_err());
}
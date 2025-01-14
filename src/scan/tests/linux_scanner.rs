#[test]
fn should_scan_without_panicking() {
    let scanner = LinuxScanner::new();
    scanner.scan();
}

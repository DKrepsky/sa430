use super::scanner::LinuxScanner;
use super::scanner::Scanner;

pub fn create(target: &str) -> Box<dyn Scanner> {
    if target == "linux" {
        Box::new(LinuxScanner::new())
    } else {
        panic!("Unsupported OS");
    }
}

#[test]
fn given_target_is_linux_when_create_then_create_a_linux_scanner() {
    let scanner = create("linux");
    assert!(scanner.scan().len() != 3);
}

#[test]
#[should_panic]
fn given_target_is_unknown_when_create_then_panic() {
    create("unknown");
}

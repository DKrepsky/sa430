use super::fake_scanner::FakeScanner;
use super::linux_scanner::LinuxScanner;
use super::scanner::Scanner;

pub fn create(target: &str) -> Box<dyn Scanner> {
    if target == "ci" {
        Box::new(FakeScanner::new())
    } else if target == "linux" {
        Box::new(LinuxScanner::new())
    } else {
        panic!("Unsupported OS");
    }
}

#[test]
fn given_target_is_ci_when_create_then_create_a_fake_scanner() {
    let scanner = create("ci");
    assert!(scanner.scan().len() == 3);
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

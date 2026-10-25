use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::predicate::str::contains;

#[test]
fn given_no_arguments_are_provided_then_print_usage_and_fail() -> Result<(), Box<dyn std::error::Error>> {
    cargo_bin_cmd!().assert().failure().stderr(contains("Usage: sa430"));

    Ok(())
}

#[test]
fn when_scan_then_list_ports() -> Result<(), Box<dyn std::error::Error>> {
    cargo_bin_cmd!()
        .arg("scan")
        .assert()
        .success()
        .stdout(contains("serial number"));

    Ok(())
}

#[test]
fn given_a_port_that_does_not_exist_when_info_then_print_error() -> Result<(), Box<dyn std::error::Error>> {
    cargo_bin_cmd!()
        .args(["info", "/this/port/does/not/exist"])
        .assert()
        .failure()
        .stderr(contains("error: serial port error"));

    Ok(())
}

#[test]
fn given_invalid_capture_params_when_capture_then_fail_before_opening_the_port(
) -> Result<(), Box<dyn std::error::Error>> {
    cargo_bin_cmd!()
        .args([
            "capture",
            "/this/port/does/not/exist",
            "--fstart",
            "434",
            "--fstop",
            "433",
            "--fstep",
            "0.1",
        ])
        .assert()
        .failure()
        .stderr(contains("fstart must be lower than fstop"));

    Ok(())
}

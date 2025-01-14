use assert_cmd::prelude::*;
use predicates::prelude::predicate::str::contains;

use std::process::Command;

const BIN_NAME: &str = "sa430";

#[test]
fn given_no_arguments_are_provided_then_return_an_error() -> Result<(), Box<dyn std::error::Error>>
{
    Command::cargo_bin(BIN_NAME)?
        .assert()
        .failure()
        .stderr(contains("No command provided, use --help for usage"));

    Ok(())
}

#[test]
fn when_scan_then_list_devices() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin(BIN_NAME)?.arg("scan").assert().success();

    Ok(())
}

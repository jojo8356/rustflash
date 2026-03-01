use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output() {
    Command::cargo_bin("rustflash")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("rustflash"));
}

#[test]
fn test_version_output() {
    Command::cargo_bin("rustflash")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rustflash"));
}

#[test]
fn test_list_command() {
    Command::cargo_bin("rustflash")
        .unwrap()
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_list_json_output() {
    Command::cargo_bin("rustflash")
        .unwrap()
        .args(["list", "--json"])
        .assert()
        .success();
}

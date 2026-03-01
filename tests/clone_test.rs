use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_clone_requires_source_arg() {
    Command::cargo_bin("rustflash")
        .unwrap()
        .arg("clone")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--source"));
}

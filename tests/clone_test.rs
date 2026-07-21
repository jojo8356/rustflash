use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_clone_requires_source_arg() {
    cargo_bin_cmd!("rustflash")
        .arg("clone")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--source"));
}

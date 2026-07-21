use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_help_output() {
    cargo_bin_cmd!("rustflash")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("rustflash"));
}

#[test]
fn test_version_output() {
    cargo_bin_cmd!("rustflash")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rustflash"));
}

#[test]
fn test_list_command() {
    cargo_bin_cmd!("rustflash")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_list_json_output() {
    let output = cargo_bin_cmd!("rustflash")
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    assert!(serde_json::from_str::<Value>(&stdout).is_ok());
}

#[test]
fn test_list_json_empty_or_devices() {
    let output = cargo_bin_cmd!("rustflash")
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let devices: Value = serde_json::from_str(&String::from_utf8_lossy(&output)).unwrap();
    assert!(devices.is_array());
}

#[test]
fn test_clone_with_invalid_compression() {
    let source = NamedTempFile::new().unwrap();
    let mut dest = NamedTempFile::new().unwrap();
    dest.write_all(&vec![0u8; 512]).unwrap();
    dest.flush().unwrap();

    cargo_bin_cmd!("rustflash")
        .args([
            "clone",
            "--yes",
            "--source",
            source.path().to_str().unwrap(),
            "--dest",
            dest.path().to_str().unwrap(),
            "--compress",
            "unknown",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown compression"));
}

#[test]
fn test_restore_rejects_corrupted_backup_file() {
    let mut corrupt_backup = NamedTempFile::new().unwrap();
    corrupt_backup.write_all(b"NOT_A_BACKUP").unwrap();
    corrupt_backup.flush().unwrap();
    let mut target = NamedTempFile::new().unwrap();
    target.write_all(&vec![0u8; 1024]).unwrap();
    target.flush().unwrap();

    cargo_bin_cmd!("rustflash")
        .args([
            "restore",
            "--yes",
            "--input",
            corrupt_backup.path().to_str().unwrap(),
            "--target",
            target.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a valid .rfb backup file"));
}

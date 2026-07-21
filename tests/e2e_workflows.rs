use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn e2e_flash_file_target_smoke() {
    let mut image = NamedTempFile::new().unwrap();
    let image_payload = vec![0x11u8; 1024 * 1024];
    image.write_all(&image_payload).unwrap();
    image.flush().unwrap();

    let mut target = NamedTempFile::new().unwrap();
    target.write_all(&vec![0u8; 2 * 1024 * 1024]).unwrap();
    target.flush().unwrap();

    cargo_bin_cmd!("rustflash")
        .args([
            "flash",
            "--yes",
            "--image",
            image.path().to_str().unwrap(),
            "--target",
            target.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Flash complete!"));

    let target_data = std::fs::read(target.path()).unwrap();
    assert_eq!(&target_data[..image_payload.len()], &image_payload[..]);
}

#[test]
fn e2e_clone_file_to_file_smoke() {
    let mut source = NamedTempFile::new().unwrap();
    let source_payload = vec![0x22u8; 512 * 1024];
    source.write_all(&source_payload).unwrap();
    source.flush().unwrap();

    let mut dest = NamedTempFile::new().unwrap();
    dest.write_all(&vec![0u8; 1024 * 1024]).unwrap();
    dest.flush().unwrap();

    cargo_bin_cmd!("rustflash")
        .args([
            "clone",
            "--yes",
            "--source",
            source.path().to_str().unwrap(),
            "--dest",
            dest.path().to_str().unwrap(),
            "--smart",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Clone complete!"));

    let cloned = std::fs::read(dest.path()).unwrap();
    assert_eq!(&cloned[..source_payload.len()], &source_payload[..]);
}

#[test]
fn e2e_backup_restore_roundtrip_smoke() {
    let mut source = NamedTempFile::new().unwrap();
    let source_payload = vec![0x33u8; 256 * 1024];
    source.write_all(&source_payload).unwrap();
    source.flush().unwrap();

    let workspace = tempdir().unwrap();
    let backup_path = workspace.path().join("source.rfb");
    let restore_target = NamedTempFile::new().unwrap();

    cargo_bin_cmd!("rustflash")
        .args([
            "backup",
            "--yes",
            "--source",
            source.path().to_str().unwrap(),
            "--output",
            backup_path.to_str().unwrap(),
            "--compress",
            "gzip",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Backup saved to"));

    cargo_bin_cmd!("rustflash")
        .args([
            "restore",
            "--yes",
            "--input",
            backup_path.to_str().unwrap(),
            "--target",
            restore_target.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Restore complete!"));

    let restored = std::fs::read(restore_target.path()).unwrap();
    assert_eq!(&restored[..source_payload.len()], &source_payload[..]);
}

#[test]
fn e2e_partition_workflow_smoke() {
    let mut device = NamedTempFile::new().unwrap();
    device.write_all(&vec![0u8; 20 * 1024 * 1024]).unwrap();
    device.flush().unwrap();
    let device_path = device.path().to_str().unwrap().to_string();

    // Create GPT partition table
    let out = cargo_bin_cmd!("rustflash")
        .args([
            "partition",
            &device_path,
            "create",
            "gpt",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Partition table created."))
        .get_output()
        .stdout
        .clone();
    assert!(String::from_utf8_lossy(&out).contains("Partition table created."));

    // Add a small partition
    cargo_bin_cmd!("rustflash")
        .args([
            "partition",
            &device_path,
            "add",
            "--fs-type",
            "ext4",
            "--size",
            "4M",
            "--label",
            "e2e_part",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Partition added."));

    // Show table
    cargo_bin_cmd!("rustflash")
        .args(["partition", &device_path, "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Device:"))
        .stdout(predicate::str::contains("e2e_part"));

    // Delete partition with confirmation
    cargo_bin_cmd!("rustflash")
        .args(["partition", &device_path, "delete", "--number", "1"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Partition deleted."));
}

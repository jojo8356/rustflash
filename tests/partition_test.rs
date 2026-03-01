use std::io::Write;
use tempfile::NamedTempFile;

use rustflash::core::partition::{PartitionManager, TableType, FsType, parse_size};

/// Helper: create a temp file of given size filled with zeros.
fn create_image(size: usize) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&vec![0u8; size]).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn test_parse_size_megabytes() {
    assert_eq!(parse_size("256M").unwrap(), 256 * 1024 * 1024);
    assert_eq!(parse_size("256m").unwrap(), 256 * 1024 * 1024);
}

#[test]
fn test_parse_size_gigabytes() {
    assert_eq!(parse_size("4G").unwrap(), 4 * 1024 * 1024 * 1024);
    assert_eq!(parse_size("1g").unwrap(), 1024 * 1024 * 1024);
}

#[test]
fn test_parse_size_terabytes() {
    assert_eq!(parse_size("1T").unwrap(), 1024u64 * 1024 * 1024 * 1024);
}

#[test]
fn test_parse_size_kilobytes() {
    assert_eq!(parse_size("512K").unwrap(), 512 * 1024);
}

#[test]
fn test_parse_size_bytes() {
    assert_eq!(parse_size("1024").unwrap(), 1024);
}

#[test]
fn test_parse_size_remaining() {
    assert_eq!(parse_size("remaining").unwrap(), 0);
    assert_eq!(parse_size("REMAINING").unwrap(), 0);
}

#[test]
fn test_parse_size_invalid() {
    assert!(parse_size("abc").is_err());
    assert!(parse_size("").is_err());
}

#[test]
fn test_fs_type_roundtrip() {
    for name in &["ext4", "fat32", "ntfs", "exfat", "swap"] {
        let fs = FsType::from_str(name);
        assert_ne!(fs, FsType::Unknown, "Failed for {name}");
        assert_eq!(FsType::from_str(fs.as_str()), fs);
    }
}

#[test]
fn test_fs_type_unknown() {
    assert_eq!(FsType::from_str("btrfs"), FsType::Unknown);
}

#[test]
fn test_create_gpt_table() {
    // Create a 10 MiB image
    let img = create_image(10 * 1024 * 1024);
    let path = img.path().to_str().unwrap();

    PartitionManager::create_table(path, TableType::Gpt).unwrap();

    let (tt, parts) = PartitionManager::read_table(path).unwrap();
    assert_eq!(tt, TableType::Gpt);
    assert!(parts.is_empty());
}

#[test]
fn test_create_mbr_table() {
    let img = create_image(1024 * 1024);
    let path = img.path().to_str().unwrap();

    PartitionManager::create_table(path, TableType::Mbr).unwrap();

    // Read the raw bytes to verify MBR signature
    let data = std::fs::read(img.path()).unwrap();
    assert_eq!(data[510], 0x55);
    assert_eq!(data[511], 0xAA);
}

#[test]
fn test_add_and_read_partition() {
    // Create a 20 MiB image with GPT
    let img = create_image(20 * 1024 * 1024);
    let path = img.path().to_str().unwrap();

    PartitionManager::create_table(path, TableType::Gpt).unwrap();

    // Add a 4 MiB partition
    PartitionManager::add_partition(path, FsType::Ext4, 4 * 1024 * 1024, Some("testpart"), &[])
        .unwrap();

    let (tt, parts) = PartitionManager::read_table(path).unwrap();
    assert_eq!(tt, TableType::Gpt);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].label.as_deref(), Some("testpart"));
}

#[test]
fn test_add_and_delete_partition() {
    let img = create_image(20 * 1024 * 1024);
    let path = img.path().to_str().unwrap();

    PartitionManager::create_table(path, TableType::Gpt).unwrap();

    // Add
    PartitionManager::add_partition(path, FsType::Fat32, 2 * 1024 * 1024, Some("todel"), &[])
        .unwrap();

    let (_, parts) = PartitionManager::read_table(path).unwrap();
    assert_eq!(parts.len(), 1);
    let num = parts[0].number;

    // Delete
    PartitionManager::delete_partition(path, num).unwrap();

    let (_, parts) = PartitionManager::read_table(path).unwrap();
    assert!(parts.is_empty());
}

#[test]
fn test_add_multiple_partitions() {
    let img = create_image(30 * 1024 * 1024);
    let path = img.path().to_str().unwrap();

    PartitionManager::create_table(path, TableType::Gpt).unwrap();

    PartitionManager::add_partition(path, FsType::Ext4, 4 * 1024 * 1024, Some("part1"), &[])
        .unwrap();
    PartitionManager::add_partition(path, FsType::Fat32, 4 * 1024 * 1024, Some("part2"), &[])
        .unwrap();

    let (_, parts) = PartitionManager::read_table(path).unwrap();
    assert_eq!(parts.len(), 2);
}

#[test]
fn test_gpt_too_small() {
    // GPT needs at least ~17 KiB
    let img = create_image(4096);
    let path = img.path().to_str().unwrap();

    let result = PartitionManager::create_table(path, TableType::Gpt);
    assert!(result.is_err());
}

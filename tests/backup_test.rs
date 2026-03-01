use std::io::Write;
use tempfile::NamedTempFile;

use rustflash::core::backup::{BackupConfig, BackupEngine, MAGIC};

async fn backup_restore_roundtrip(compression: &str) {
    // Create source data
    let mut src = NamedTempFile::new().unwrap();
    let test_data = vec![0xCCu8; 256 * 1024];
    src.write_all(&test_data).unwrap();
    src.flush().unwrap();

    // Output .rfb file
    let output = tempfile::Builder::new()
        .suffix(".rfb")
        .tempfile()
        .unwrap();

    // Create backup
    let config = BackupConfig {
        block_size: 64 * 1024,
        compression: compression.into(),
        compression_level: 3,
        smart: false,
    };
    let engine = BackupEngine::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move {
        while rx.recv().await.is_some() {}
    });

    engine
        .create_backup(
            src.path().to_str().unwrap(),
            output.path().to_str().unwrap(),
            tx,
        )
        .await
        .unwrap();

    // Verify .rfb file starts with magic
    let rfb_data = std::fs::read(output.path()).unwrap();
    assert_eq!(&rfb_data[..8], MAGIC);

    // Read header
    let header = BackupEngine::read_header(output.path().to_str().unwrap()).unwrap();
    assert_eq!(header.version, 1);
    assert_eq!(header.source_size, test_data.len() as u64);
    assert_eq!(header.compression, compression);
    assert!(!header.checksum.is_empty());
    assert_ne!(header.checksum, "0".repeat(64));

    // Restore
    let mut restore_target = NamedTempFile::new().unwrap();
    restore_target.write_all(&vec![0u8; 512 * 1024]).unwrap();
    restore_target.flush().unwrap();

    let restore_config = BackupConfig::default();
    let restore_engine = BackupEngine::new(restore_config);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move {
        while rx2.recv().await.is_some() {}
    });

    restore_engine
        .restore_backup(
            output.path().to_str().unwrap(),
            restore_target.path().to_str().unwrap(),
            tx2,
        )
        .await
        .unwrap();

    // Verify restored data matches original
    let restored = std::fs::read(restore_target.path()).unwrap();
    assert_eq!(&restored[..test_data.len()], &test_data[..]);
}

#[tokio::test]
async fn test_backup_roundtrip_zstd() {
    backup_restore_roundtrip("zstd").await;
}

#[tokio::test]
async fn test_backup_roundtrip_gzip() {
    backup_restore_roundtrip("gzip").await;
}

#[tokio::test]
async fn test_backup_roundtrip_xz() {
    backup_restore_roundtrip("xz").await;
}

#[tokio::test]
async fn test_backup_invalid_magic() {
    let mut bad_file = NamedTempFile::new().unwrap();
    bad_file.write_all(b"NOT_RFLASH_DATA").unwrap();
    bad_file.flush().unwrap();

    let result = BackupEngine::read_header(bad_file.path().to_str().unwrap());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("invalid magic"));
}

#[tokio::test]
async fn test_backup_read_header() {
    // Create a valid backup first
    let mut src = NamedTempFile::new().unwrap();
    src.write_all(&vec![0xEEu8; 64 * 1024]).unwrap();
    src.flush().unwrap();

    let output = tempfile::Builder::new()
        .suffix(".rfb")
        .tempfile()
        .unwrap();

    let config = BackupConfig {
        block_size: 32 * 1024,
        compression: "zstd".into(),
        compression_level: 1,
        smart: false,
    };

    let engine = BackupEngine::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move {
        while rx.recv().await.is_some() {}
    });

    engine
        .create_backup(
            src.path().to_str().unwrap(),
            output.path().to_str().unwrap(),
            tx,
        )
        .await
        .unwrap();

    // Read header independently
    let header = BackupEngine::read_header(output.path().to_str().unwrap()).unwrap();
    assert_eq!(header.version, 1);
    assert_eq!(header.block_size, 32 * 1024);
    assert_eq!(header.compression, "zstd");
    assert_eq!(header.source_size, 64 * 1024);
    assert_eq!(header.hash_algorithm, "sha256");
}

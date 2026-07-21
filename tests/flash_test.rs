use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_flash_requires_image_arg() {
    cargo_bin_cmd!("rustflash")
        .arg("flash")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--image"));
}

#[test]
fn test_flash_requires_target_arg() {
    cargo_bin_cmd!("rustflash")
        .args(["flash", "--image", "test.iso"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--target"));
}

#[tokio::test]
async fn test_flash_raw_image_to_file() {
    // Create a small test image (1 MiB of 0xAB)
    let mut img = NamedTempFile::new().unwrap();
    let test_data = vec![0xABu8; 1024 * 1024];
    img.write_all(&test_data).unwrap();
    img.flush().unwrap();

    // Create target file (pre-sized with zeros)
    let mut target = NamedTempFile::new().unwrap();
    target.write_all(&vec![0u8; 2 * 1024 * 1024]).unwrap();
    target.flush().unwrap();

    let config = rustflash::core::flasher::FlashConfig {
        block_size: 64 * 1024,
        verify: false,
        auto_unmount: true,
    };

    let flasher = rustflash::core::flasher::Flasher::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    flasher
        .flash(img.path(), target.path().to_str().unwrap(), tx)
        .await
        .unwrap();

    // Verify the target file starts with the image data
    let target_data = std::fs::read(target.path()).unwrap();
    assert_eq!(&target_data[..test_data.len()], &test_data[..]);
}

#[tokio::test]
async fn test_flash_with_verification() {
    let mut img = NamedTempFile::new().unwrap();
    let test_data = vec![0xCDu8; 512 * 1024];
    img.write_all(&test_data).unwrap();
    img.flush().unwrap();

    let mut target = NamedTempFile::new().unwrap();
    target.write_all(&vec![0u8; 1024 * 1024]).unwrap();
    target.flush().unwrap();

    let config = rustflash::core::flasher::FlashConfig {
        block_size: 64 * 1024,
        verify: true,
        auto_unmount: true,
    };

    let flasher = rustflash::core::flasher::Flasher::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    flasher
        .flash(img.path(), target.path().to_str().unwrap(), tx)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_flash_gzip_image_to_file() {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let test_data = vec![0xEFu8; 512 * 1024];

    // Create gzip-compressed image
    let mut img = tempfile::Builder::new()
        .suffix(".img.gz")
        .tempfile()
        .unwrap();
    {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&test_data).unwrap();
        let compressed = encoder.finish().unwrap();
        img.write_all(&compressed).unwrap();
        img.flush().unwrap();
    }

    // Create target
    let mut target = NamedTempFile::new().unwrap();
    target.write_all(&vec![0u8; 1024 * 1024]).unwrap();
    target.flush().unwrap();

    let config = rustflash::core::flasher::FlashConfig {
        block_size: 64 * 1024,
        verify: false,
        auto_unmount: true,
    };

    let flasher = rustflash::core::flasher::Flasher::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    flasher
        .flash(img.path(), target.path().to_str().unwrap(), tx)
        .await
        .unwrap();

    let target_data = std::fs::read(target.path()).unwrap();
    assert_eq!(&target_data[..test_data.len()], &test_data[..]);
}

#[test]
fn test_verify_file_checksum_sha256() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"hello world").unwrap();
    f.flush().unwrap();

    let expected = "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
    let result = rustflash::core::verify::verify_file_checksum(f.path(), expected).unwrap();
    assert!(result);
}

#[test]
fn test_verify_file_checksum_wrong() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"hello world").unwrap();
    f.flush().unwrap();

    let result = rustflash::core::verify::verify_file_checksum(
        f.path(),
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();
    assert!(!result);
}

#[test]
fn test_verify_file_checksum_blake3() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"hello world").unwrap();
    f.flush().unwrap();

    let expected_hash = blake3::hash(b"hello world").to_hex().to_string();
    let expected = format!("blake3:{expected_hash}");
    let result = rustflash::core::verify::verify_file_checksum(f.path(), &expected).unwrap();
    assert!(result);
}

// ── Phase 2 tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_flash_zip_image_to_file() {
    use std::io::Cursor;

    let test_data = vec![0xBBu8; 256 * 1024];

    // Create a ZIP archive containing an .img file
    let mut zip_buf: Vec<u8> = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(Cursor::new(&mut zip_buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer.start_file("disk.img", options).unwrap();
        writer.write_all(&test_data).unwrap();
        writer.finish().unwrap();
    }

    let mut img = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
    img.write_all(&zip_buf).unwrap();
    img.flush().unwrap();

    // Create target
    let mut target = NamedTempFile::new().unwrap();
    target.write_all(&vec![0u8; 512 * 1024]).unwrap();
    target.flush().unwrap();

    let config = rustflash::core::flasher::FlashConfig {
        block_size: 64 * 1024,
        verify: false,
        auto_unmount: true,
    };

    let flasher = rustflash::core::flasher::Flasher::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    flasher
        .flash(img.path(), target.path().to_str().unwrap(), tx)
        .await
        .unwrap();

    let target_data = std::fs::read(target.path()).unwrap();
    assert_eq!(&target_data[..test_data.len()], &test_data[..]);
}

#[tokio::test]
async fn test_multi_flash_parallel() {
    // Create a small test image
    let mut img = NamedTempFile::new().unwrap();
    let test_data = vec![0xDDu8; 128 * 1024];
    img.write_all(&test_data).unwrap();
    img.flush().unwrap();

    // Create two target files
    let mut target1 = NamedTempFile::new().unwrap();
    target1.write_all(&vec![0u8; 256 * 1024]).unwrap();
    target1.flush().unwrap();

    let mut target2 = NamedTempFile::new().unwrap();
    target2.write_all(&vec![0u8; 256 * 1024]).unwrap();
    target2.flush().unwrap();

    let config = rustflash::core::flasher::FlashConfig {
        block_size: 32 * 1024,
        verify: false,
        auto_unmount: true,
    };

    let flasher = rustflash::core::flasher::Flasher::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(128);

    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    let targets = vec![
        target1.path().to_str().unwrap().to_string(),
        target2.path().to_str().unwrap().to_string(),
    ];

    let results = flasher.flash_multi(img.path(), &targets, tx).await;

    assert_eq!(results.len(), 2);
    for r in &results {
        assert!(r.success, "Flash failed for {}: {:?}", r.device, r.error);
    }

    // Verify both targets have the correct data
    let data1 = std::fs::read(target1.path()).unwrap();
    assert_eq!(&data1[..test_data.len()], &test_data[..]);

    let data2 = std::fs::read(target2.path()).unwrap();
    assert_eq!(&data2[..test_data.len()], &test_data[..]);
}

#[tokio::test]
async fn test_clone_raw_file_to_file() {
    // Create source with pattern
    let mut src = NamedTempFile::new().unwrap();
    let test_data = vec![0xAAu8; 256 * 1024];
    src.write_all(&test_data).unwrap();
    src.flush().unwrap();

    // Create dest (pre-sized)
    let mut dest = NamedTempFile::new().unwrap();
    dest.write_all(&vec![0u8; 512 * 1024]).unwrap();
    dest.flush().unwrap();

    let config = rustflash::core::cloner::CloneConfig {
        mode: rustflash::core::cloner::CloneMode::Raw,
        block_size: 64 * 1024,
        verify: false,
        compression: None,
    };

    let cloner = rustflash::core::cloner::Cloner::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    cloner
        .clone_device(
            src.path().to_str().unwrap(),
            dest.path().to_str().unwrap(),
            tx,
        )
        .await
        .unwrap();

    let dest_data = std::fs::read(dest.path()).unwrap();
    assert_eq!(&dest_data[..test_data.len()], &test_data[..]);
}

#[tokio::test]
async fn test_clone_with_gzip_compression() {
    use std::io::Read;

    let mut src = NamedTempFile::new().unwrap();
    let test_data = vec![0xBBu8; 128 * 1024];
    src.write_all(&test_data).unwrap();
    src.flush().unwrap();

    let dest = tempfile::Builder::new()
        .suffix(".img.gz")
        .tempfile()
        .unwrap();

    let config = rustflash::core::cloner::CloneConfig {
        mode: rustflash::core::cloner::CloneMode::Raw,
        block_size: 64 * 1024,
        verify: false,
        compression: Some(rustflash::core::cloner::CompressionType::Gzip),
    };

    let cloner = rustflash::core::cloner::Cloner::new(config);
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    cloner
        .clone_device(
            src.path().to_str().unwrap(),
            dest.path().to_str().unwrap(),
            tx,
        )
        .await
        .unwrap();

    // Decompress and verify
    let compressed = std::fs::read(dest.path()).unwrap();
    let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    assert_eq!(decompressed, test_data);
}

#[test]
fn test_decompress_detect_format() {
    use rustflash::io::decompress::{ImageFormat, detect_format};
    use std::path::Path;

    assert_eq!(detect_format(Path::new("test.img")), ImageFormat::Raw);
    assert_eq!(detect_format(Path::new("test.img.gz")), ImageFormat::Gzip);
    assert_eq!(detect_format(Path::new("test.img.xz")), ImageFormat::Xz);
    assert_eq!(detect_format(Path::new("test.img.zst")), ImageFormat::Zstd);
    assert_eq!(detect_format(Path::new("test.img.bz2")), ImageFormat::Bzip2);
    assert_eq!(detect_format(Path::new("test.zip")), ImageFormat::Zip);
    assert_eq!(detect_format(Path::new("test.iso")), ImageFormat::Raw);
}

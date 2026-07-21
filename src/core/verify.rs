use std::io::Read;
use std::path::Path;

use sha2::Digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `HashAlgorithm`
pub enum HashAlgorithm {
    /// Variante d'énumération `Sha256` du type énuméré.
    Sha256,
    /// Variante d'énumération `Sha512` du type énuméré.
    Sha512,
    /// Variante d'énumération `Md5` du type énuméré.
    Md5,
    /// Variante d'énumération `Blake3` du type énuméré.
    Blake3,
}

impl HashAlgorithm {
    /// Fonction publique `from_prefix`
    pub fn from_prefix(s: &str) -> Option<(Self, &str)> {
        if let Some(hash) = s.strip_prefix("sha256:") {
            Some((Self::Sha256, hash))
        } else if let Some(hash) = s.strip_prefix("sha512:") {
            Some((Self::Sha512, hash))
        } else if let Some(hash) = s.strip_prefix("md5:") {
            Some((Self::Md5, hash))
        } else if let Some(hash) = s.strip_prefix("blake3:") {
            Some((Self::Blake3, hash))
        } else {
            None
        }
    }
}

/// Verify a file's checksum using streaming reads (no full file in memory).
pub fn verify_file_checksum(path: &Path, expected: &str) -> anyhow::Result<bool> {
    let Some((algo, expected_hash)) = HashAlgorithm::from_prefix(expected) else {
        anyhow::bail!(
            "Invalid checksum format. Expected: sha256:HASH, sha512:HASH, md5:HASH, or blake3:HASH"
        );
    };

    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let computed = compute_hash_streaming(&mut reader, algo)?;

    Ok(computed == expected_hash)
}

/// Compute hash of in-memory data.
pub fn compute_hash(data: &[u8], algo: HashAlgorithm) -> String {
    match algo {
        HashAlgorithm::Sha256 => hex::encode(sha2::Sha256::digest(data)),
        HashAlgorithm::Sha512 => hex::encode(sha2::Sha512::digest(data)),
        HashAlgorithm::Md5 => hex::encode(md5::Md5::digest(data)),
        HashAlgorithm::Blake3 => blake3::hash(data).to_hex().to_string(),
    }
}

fn compute_hash_streaming(reader: &mut dyn Read, algo: HashAlgorithm) -> anyhow::Result<String> {
    let mut buf = vec![0u8; 64 * 1024];
    match algo {
        HashAlgorithm::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hex::encode(hasher.finalize()))
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hex::encode(hasher.finalize()))
        }
        HashAlgorithm::Md5 => {
            let mut hasher = md5::Md5::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                Digest::update(&mut hasher, &buf[..n]);
            }
            Ok(hex::encode(hasher.finalize()))
        }
        HashAlgorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hasher.finalize().to_hex().to_string())
        }
    }
}

/// Verify device contents match the image by block-by-block comparison.
pub async fn verify_device_matches_image(
    device_path: &str,
    image_path: &Path,
    block_size: usize,
) -> anyhow::Result<bool> {
    tracing::info!(
        device = device_path,
        image = %image_path.display(),
        "Verifying device matches image"
    );

    let device_path_owned = device_path.to_string();
    let image_path_owned = image_path.to_owned();

    tokio::task::spawn_blocking(move || {
        verify_device_matches_image_sync(&device_path_owned, &image_path_owned, block_size)
    })
    .await?
}

fn verify_device_matches_image_sync(
    device_path: &str,
    image_path: &Path,
    block_size: usize,
) -> anyhow::Result<bool> {
    let mut image_reader = crate::io::decompress::open_image(image_path)?;
    let mut device_file = std::fs::File::open(device_path)?;

    let mut img_buf = vec![0u8; block_size];
    let mut dev_buf = vec![0u8; block_size];
    let mut offset: u64 = 0;

    loop {
        let img_n = crate::io::read_full(&mut image_reader, &mut img_buf)?;
        if img_n == 0 {
            break;
        }
        let dev_n = crate::io::read_full(&mut device_file, &mut dev_buf[..img_n])?;
        if dev_n != img_n {
            tracing::error!(offset, img_n, dev_n, "Device shorter than image");
            return Ok(false);
        }
        if img_buf[..img_n] != dev_buf[..img_n] {
            tracing::error!(offset, "Block mismatch during verification");
            return Ok(false);
        }
        offset += img_n as u64;
    }

    tracing::info!(bytes_verified = offset, "Verification passed");
    Ok(true)
}

use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tokio::sync::mpsc;

pub const MAGIC: &[u8; 8] = b"RFLASH\x01\x00";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupHeader {
    pub version: u32,
    pub created: DateTime<Utc>,
    pub source_size: u64,
    pub block_size: u32,
    pub compression: String,
    pub hash_algorithm: String,
    pub partition_table: Option<String>,
    pub source_device: Option<String>,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub block_size: usize,
    pub compression: String,
    pub compression_level: u32,
    pub smart: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            block_size: 4 * 1024 * 1024,
            compression: "zstd".into(),
            compression_level: 3,
            smart: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackupProgress {
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub phase: BackupPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupPhase {
    Analyzing,
    Reading,
    Compressing,
    Done,
    Failed,
}

pub struct BackupEngine {
    config: BackupConfig,
}

impl BackupEngine {
    pub fn new(config: BackupConfig) -> Self {
        Self { config }
    }

    /// Read only the header from an .rfb file (for dry-run / display).
    pub fn read_header(input_path: &str) -> anyhow::Result<BackupHeader> {
        let file = std::fs::File::open(input_path)?;
        let mut reader = BufReader::new(file);

        // Read and validate magic
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC {
            anyhow::bail!("Not a valid .rfb backup file (invalid magic)");
        }

        // Read header length
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let header_len = u32::from_le_bytes(len_buf) as usize;

        // Read header JSON
        let mut header_buf = vec![0u8; header_len];
        reader.read_exact(&mut header_buf)?;
        let header: BackupHeader = serde_json::from_slice(&header_buf)?;

        Ok(header)
    }

    pub async fn create_backup(
        &self,
        source: &str,
        output_path: &str,
        progress_tx: mpsc::Sender<BackupProgress>,
    ) -> anyhow::Result<()> {
        tracing::info!(source, output = output_path, "Creating backup");

        let block_size = self.config.block_size;
        let compression = self.config.compression.clone();
        let compression_level = self.config.compression_level;
        let source_owned = source.to_string();
        let output_owned = output_path.to_string();

        let _ = progress_tx
            .send(BackupProgress {
                bytes_processed: 0,
                total_bytes: 0,
                phase: BackupPhase::Analyzing,
            })
            .await;

        let ptx = progress_tx.clone();

        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            // Open source and get size
            let mut source_file = std::fs::File::open(&source_owned)?;
            let source_size = match source_file.seek(SeekFrom::End(0)) {
                Ok(size) if size > 0 => {
                    source_file.seek(SeekFrom::Start(0))?;
                    size
                }
                _ => {
                    source_file.seek(SeekFrom::Start(0))?;
                    std::fs::metadata(&source_owned)?.len()
                }
            };

            // Build header with placeholder checksum (64 zeros for SHA-256)
            let placeholder_checksum = "0".repeat(64);
            let header = BackupHeader {
                version: 1,
                created: Utc::now(),
                source_size,
                block_size: block_size as u32,
                compression: compression.clone(),
                hash_algorithm: "sha256".into(),
                partition_table: None,
                source_device: Some(source_owned.clone()),
                checksum: placeholder_checksum,
            };

            let header_json = serde_json::to_vec(&header)?;
            let header_json_len = header_json.len();

            // Open output
            let out_file = std::fs::OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .truncate(true)
                .open(&output_owned)?;
            let mut writer = BufWriter::new(out_file);

            // Track global hash for footer
            let mut global_hasher = sha2::Sha256::new();

            // Write MAGIC
            writer.write_all(MAGIC)?;
            global_hasher.update(MAGIC);

            // Write header length
            let header_len_bytes = (header_json_len as u32).to_le_bytes();
            writer.write_all(&header_len_bytes)?;
            global_hasher.update(&header_len_bytes);

            // Write header JSON
            writer.write_all(&header_json)?;
            global_hasher.update(&header_json);

            let _ = ptx.blocking_send(BackupProgress {
                bytes_processed: 0,
                total_bytes: source_size,
                phase: BackupPhase::Reading,
            });

            // Read/compress/write loop
            let mut buf = vec![0u8; block_size];
            let mut raw_hasher = sha2::Sha256::new();
            let mut bytes_processed: u64 = 0;

            loop {
                let n = crate::io::read_full(&mut source_file, &mut buf)?;
                if n == 0 {
                    break;
                }

                raw_hasher.update(&buf[..n]);

                let compressed = compress_block(&buf[..n], &compression, compression_level)?;

                let chunk_len = compressed.len() as u32;
                let chunk_len_bytes = chunk_len.to_le_bytes();
                writer.write_all(&chunk_len_bytes)?;
                writer.write_all(&compressed)?;
                global_hasher.update(&chunk_len_bytes);
                global_hasher.update(&compressed);

                bytes_processed += n as u64;
                let _ = ptx.blocking_send(BackupProgress {
                    bytes_processed,
                    total_bytes: source_size,
                    phase: BackupPhase::Compressing,
                });
            }

            // Write zero sentinel
            let sentinel = 0u32.to_le_bytes();
            writer.write_all(&sentinel)?;
            global_hasher.update(&sentinel);

            // Flush the BufWriter to get the inner file
            writer.flush()?;
            let mut out_file = writer.into_inner()?;

            // Rewrite header with real checksum
            let raw_checksum = hex::encode(raw_hasher.finalize());
            let final_header = BackupHeader {
                checksum: raw_checksum,
                ..header
            };
            let final_header_json = serde_json::to_vec(&final_header)?;

            // The JSON length must match (placeholder was same length as real SHA-256 hex)
            assert_eq!(
                final_header_json.len(),
                header_json_len,
                "Header JSON length changed after checksum update"
            );

            // Seek to header position (after MAGIC + 4-byte length)
            out_file.seek(SeekFrom::Start(12))?;
            out_file.write_all(&final_header_json)?;

            // Seek to end and write footer hash
            out_file.seek(SeekFrom::End(0))?;
            // Recompute global hash with the real header
            // Since only the header content changed (same length), we need to recompute
            // Actually, recomputing from scratch is simpler and correct:
            let footer_hash = {
                out_file.seek(SeekFrom::Start(0))?;
                let mut hasher = sha2::Sha256::new();
                let mut fbuf = vec![0u8; 64 * 1024];
                loop {
                    let n = out_file.read(&mut fbuf)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&fbuf[..n]);
                }
                hasher.finalize()
            };
            out_file.write_all(&footer_hash)?;
            out_file.sync_all()?;

            let _ = ptx.blocking_send(BackupProgress {
                bytes_processed: source_size,
                total_bytes: source_size,
                phase: BackupPhase::Done,
            });

            tracing::info!(
                source = source_owned,
                output = output_owned,
                source_size,
                "Backup complete"
            );

            Ok(())
        })
        .await??;

        Ok(())
    }

    pub async fn restore_backup(
        &self,
        input_path: &str,
        target: &str,
        progress_tx: mpsc::Sender<BackupProgress>,
    ) -> anyhow::Result<()> {
        tracing::info!(input = input_path, target, "Restoring backup");

        let input_owned = input_path.to_string();
        let target_owned = target.to_string();

        let _ = progress_tx
            .send(BackupProgress {
                bytes_processed: 0,
                total_bytes: 0,
                phase: BackupPhase::Analyzing,
            })
            .await;

        let ptx = progress_tx.clone();

        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let file = std::fs::File::open(&input_owned)?;
            let mut reader = BufReader::new(file);

            // Read and validate magic
            let mut magic = [0u8; 8];
            reader.read_exact(&mut magic)?;
            if &magic != MAGIC {
                anyhow::bail!("Not a valid .rfb backup file (invalid magic)");
            }

            // Read header
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let header_len = u32::from_le_bytes(len_buf) as usize;

            let mut header_buf = vec![0u8; header_len];
            reader.read_exact(&mut header_buf)?;
            let header: BackupHeader = serde_json::from_slice(&header_buf)?;

            if header.version != 1 {
                anyhow::bail!("Unsupported backup version: {}", header.version);
            }

            tracing::info!(
                source_size = header.source_size,
                compression = %header.compression,
                source_device = ?header.source_device,
                "Backup header parsed"
            );

            let _ = ptx.blocking_send(BackupProgress {
                bytes_processed: 0,
                total_bytes: header.source_size,
                phase: BackupPhase::Reading,
            });

            // Open target for writing
            let mut target_file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&target_owned)?;

            let mut raw_hasher = sha2::Sha256::new();
            let mut bytes_written: u64 = 0;

            // Read chunks and decompress
            loop {
                let mut chunk_len_buf = [0u8; 4];
                reader.read_exact(&mut chunk_len_buf)?;
                let chunk_len = u32::from_le_bytes(chunk_len_buf);

                if chunk_len == 0 {
                    break; // sentinel
                }

                let mut compressed = vec![0u8; chunk_len as usize];
                reader.read_exact(&mut compressed)?;

                let raw = decompress_block(&compressed, &header.compression)?;
                raw_hasher.update(&raw);
                target_file.write_all(&raw)?;
                bytes_written += raw.len() as u64;

                let _ = ptx.blocking_send(BackupProgress {
                    bytes_processed: bytes_written,
                    total_bytes: header.source_size,
                    phase: BackupPhase::Reading,
                });
            }

            target_file.flush()?;
            target_file.sync_all()?;

            // Verify raw data checksum
            let computed_checksum = hex::encode(raw_hasher.finalize());
            if computed_checksum != header.checksum {
                anyhow::bail!(
                    "Backup integrity check failed: expected checksum {}, got {}",
                    header.checksum,
                    computed_checksum
                );
            }

            tracing::info!(bytes_written, "Restore complete, checksum verified");

            let _ = ptx.blocking_send(BackupProgress {
                bytes_processed: bytes_written,
                total_bytes: header.source_size,
                phase: BackupPhase::Done,
            });

            Ok(())
        })
        .await??;

        Ok(())
    }
}

fn compress_block(data: &[u8], compression: &str, level: u32) -> anyhow::Result<Vec<u8>> {
    match compression {
        "gzip" | "gz" => {
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(level));
            encoder.write_all(data)?;
            Ok(encoder.finish()?)
        }
        "xz" => {
            let mut encoder = xz2::write::XzEncoder::new(Vec::new(), level);
            encoder.write_all(data)?;
            Ok(encoder.finish()?)
        }
        "zstd" | "zst" => {
            let compressed = zstd::stream::encode_all(Cursor::new(data), level as i32)?;
            Ok(compressed)
        }
        other => anyhow::bail!("Unknown compression algorithm: {other}"),
    }
}

fn decompress_block(data: &[u8], compression: &str) -> anyhow::Result<Vec<u8>> {
    match compression {
        "gzip" | "gz" => {
            let mut decoder = flate2::read::GzDecoder::new(Cursor::new(data));
            let mut out = Vec::new();
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        "xz" => {
            let mut decoder = xz2::read::XzDecoder::new(Cursor::new(data));
            let mut out = Vec::new();
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        "zstd" | "zst" => {
            let out = zstd::stream::decode_all(Cursor::new(data))?;
            Ok(out)
        }
        other => anyhow::bail!("Unknown compression algorithm: {other}"),
    }
}

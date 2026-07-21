use std::io::{Read, Seek, SeekFrom, Write};

use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `CloneMode`
pub enum CloneMode {
    /// Variante d'énumération `Raw` du type énuméré.
    Raw,
    /// Variante d'énumération `Smart` du type énuméré.
    Smart,
}

#[derive(Debug, Clone)]
/// Structure publique `CloneConfig`
pub struct CloneConfig {
    /// Champ public `mode` de la structure correspondante.
    pub mode: CloneMode,
    /// Champ public `block_size` de la structure correspondante.
    pub block_size: usize,
    /// Champ public `verify` de la structure correspondante.
    pub verify: bool,
    /// Champ public `compression` de la structure correspondante.
    pub compression: Option<CompressionType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `CompressionType`
pub enum CompressionType {
    /// Variante d'énumération `Gzip` du type énuméré.
    Gzip,
    /// Variante d'énumération `Xz` du type énuméré.
    Xz,
    /// Variante d'énumération `Zstd` du type énuméré.
    Zstd,
}

#[derive(Debug, Clone)]
/// Structure publique `CloneProgress`
pub struct CloneProgress {
    /// Champ public `bytes_copied` de la structure correspondante.
    pub bytes_copied: u64,
    /// Champ public `total_bytes` de la structure correspondante.
    pub total_bytes: u64,
    /// Champ public `phase` de la structure correspondante.
    pub phase: ClonePhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `ClonePhase`
pub enum ClonePhase {
    /// Variante d'énumération `Analyzing` du type énuméré.
    Analyzing,
    /// Variante d'énumération `Copying` du type énuméré.
    Copying,
    /// Variante d'énumération `Verifying` du type énuméré.
    Verifying,
    /// Variante d'énumération `Done` du type énuméré.
    Done,
    /// Variante d'énumération `Failed` du type énuméré.
    Failed,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            mode: CloneMode::Raw,
            block_size: 4 * 1024 * 1024,
            verify: true,
            compression: None,
        }
    }
}

/// Structure publique `Cloner`
pub struct Cloner {
    config: CloneConfig,
}

impl Cloner {
    /// Fonction publique `new`
    pub fn new(config: CloneConfig) -> Self {
        Self { config }
    }

    /// Fonction publique `clone_device`
    pub async fn clone_device(
        &self,
        source: &str,
        dest: &str,
        progress_tx: mpsc::Sender<CloneProgress>,
    ) -> anyhow::Result<()> {
        tracing::info!(source, dest, mode = ?self.config.mode, "Starting clone");

        if self.config.mode == CloneMode::Smart {
            tracing::warn!("Smart clone not yet implemented; falling back to raw clone");
        }

        let block_size = self.config.block_size;
        let verify = self.config.verify;
        let compression = self.config.compression;
        let source_owned = source.to_string();
        let dest_owned = dest.to_string();

        let _ = progress_tx
            .send(CloneProgress {
                bytes_copied: 0,
                total_bytes: 0,
                phase: ClonePhase::Analyzing,
            })
            .await;

        let ptx = progress_tx.clone();
        let src = source_owned.clone();
        let dst = dest_owned.clone();

        let bytes_copied = tokio::task::spawn_blocking(move || -> anyhow::Result<u64> {
            // Get source size
            let mut source_file = std::fs::File::open(&src)?;
            let total_bytes = match source_file.seek(SeekFrom::End(0)) {
                Ok(size) if size > 0 => {
                    source_file.seek(SeekFrom::Start(0))?;
                    size
                }
                _ => {
                    source_file.seek(SeekFrom::Start(0))?;
                    std::fs::metadata(&src)?.len()
                }
            };

            let _ = ptx.blocking_send(CloneProgress {
                bytes_copied: 0,
                total_bytes,
                phase: ClonePhase::Copying,
            });

            // Open dest and wrap in optional compressor
            let dest_file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&dst)?;

            let mut buf = vec![0u8; block_size];

            let written: u64 = match compression {
                None => {
                    let mut dest_writer = dest_file;
                    let w = copy_blocks(
                        &mut source_file,
                        &mut dest_writer,
                        &mut buf,
                        total_bytes,
                        &ptx,
                    )?;
                    dest_writer.flush()?;
                    dest_writer.sync_all()?;
                    w
                }
                Some(CompressionType::Gzip) => {
                    let mut encoder =
                        flate2::write::GzEncoder::new(dest_file, flate2::Compression::default());
                    let w =
                        copy_blocks(&mut source_file, &mut encoder, &mut buf, total_bytes, &ptx)?;
                    let inner = encoder.finish()?;
                    inner.sync_all()?;
                    w
                }
                Some(CompressionType::Xz) => {
                    let mut encoder = xz2::write::XzEncoder::new(dest_file, 6);
                    let w =
                        copy_blocks(&mut source_file, &mut encoder, &mut buf, total_bytes, &ptx)?;
                    let inner = encoder.finish()?;
                    inner.sync_all()?;
                    w
                }
                Some(CompressionType::Zstd) => {
                    let mut encoder = zstd::stream::write::Encoder::new(dest_file, 3)?;
                    let w =
                        copy_blocks(&mut source_file, &mut encoder, &mut buf, total_bytes, &ptx)?;
                    let inner = encoder.finish()?;
                    inner.sync_all()?;
                    w
                }
            };

            Ok(written)
        })
        .await??;

        tracing::info!(bytes_copied, "Clone copy complete");

        // Verification pass (only for raw uncompressed clones)
        if verify && compression.is_none() {
            let _ = progress_tx
                .send(CloneProgress {
                    bytes_copied: 0,
                    total_bytes: bytes_copied,
                    phase: ClonePhase::Verifying,
                })
                .await;

            let src = source_owned.clone();
            let dst = dest_owned.clone();
            let matches = tokio::task::spawn_blocking(move || -> anyhow::Result<bool> {
                let mut src_file = std::fs::File::open(&src)?;
                let mut dst_file = std::fs::File::open(&dst)?;
                let mut src_buf = vec![0u8; block_size];
                let mut dst_buf = vec![0u8; block_size];

                loop {
                    let sn = crate::io::read_full(&mut src_file, &mut src_buf)?;
                    if sn == 0 {
                        break;
                    }
                    let dn = crate::io::read_full(&mut dst_file, &mut dst_buf[..sn])?;
                    if dn != sn || src_buf[..sn] != dst_buf[..sn] {
                        return Ok(false);
                    }
                }
                Ok(true)
            })
            .await??;

            if !matches {
                anyhow::bail!("Clone verification failed: source and destination do not match");
            }

            tracing::info!("Clone verification passed");
        }

        let _ = progress_tx
            .send(CloneProgress {
                bytes_copied,
                total_bytes: bytes_copied,
                phase: ClonePhase::Done,
            })
            .await;

        Ok(())
    }
}

fn copy_blocks(
    source: &mut dyn Read,
    dest: &mut dyn Write,
    buf: &mut [u8],
    total_bytes: u64,
    ptx: &mpsc::Sender<CloneProgress>,
) -> anyhow::Result<u64> {
    let mut written: u64 = 0;
    loop {
        let n = crate::io::read_full(source, buf)?;
        if n == 0 {
            break;
        }
        dest.write_all(&buf[..n])?;
        written += n as u64;
        let _ = ptx.blocking_send(CloneProgress {
            bytes_copied: written,
            total_bytes,
            phase: ClonePhase::Copying,
        });
    }
    Ok(written)
}

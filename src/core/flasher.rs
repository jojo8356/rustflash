use std::io::Write;
use std::path::Path;

use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct FlashConfig {
    pub block_size: usize,
    pub verify: bool,
    pub auto_unmount: bool,
}

impl Default for FlashConfig {
    fn default() -> Self {
        Self {
            block_size: 4 * 1024 * 1024,
            verify: true,
            auto_unmount: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlashProgress {
    pub device_index: usize,
    pub device_name: String,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub phase: FlashPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashPhase {
    Preparing,
    Writing,
    Verifying,
    Done,
    Failed,
}

/// Result of a multi-flash operation per device.
#[derive(Debug)]
pub struct FlashResult {
    pub device: String,
    pub success: bool,
    pub bytes_written: u64,
    pub error: Option<String>,
}

pub struct Flasher {
    config: FlashConfig,
}

impl Flasher {
    pub fn new(config: FlashConfig) -> Self {
        Self { config }
    }

    /// Flash an image to a single target device.
    pub async fn flash(
        &self,
        image_path: &Path,
        target_device: &str,
        progress_tx: mpsc::Sender<FlashProgress>,
    ) -> anyhow::Result<()> {
        self.flash_single(image_path, target_device, 0, progress_tx)
            .await
    }

    /// Flash an image to multiple target devices in parallel.
    pub async fn flash_multi(
        &self,
        image_path: &Path,
        targets: &[String],
        progress_tx: mpsc::Sender<FlashProgress>,
    ) -> Vec<FlashResult> {
        tracing::info!(
            image = %image_path.display(),
            targets = ?targets,
            "Starting multi-flash to {} devices",
            targets.len()
        );

        let mut handles = Vec::new();

        for (idx, target) in targets.iter().enumerate() {
            let config = self.config.clone();
            let img = image_path.to_owned();
            let tgt = target.clone();
            let ptx = progress_tx.clone();

            let handle = tokio::spawn(async move {
                let flasher = Flasher::new(config);
                match flasher.flash_single(&img, &tgt, idx, ptx.clone()).await {
                    Ok(()) => FlashResult {
                        device: tgt,
                        success: true,
                        bytes_written: 0, // could track, but Done phase has it
                        error: None,
                    },
                    Err(e) => {
                        let _ = ptx
                            .send(FlashProgress {
                                device_index: idx,
                                device_name: tgt.clone(),
                                bytes_written: 0,
                                total_bytes: 0,
                                phase: FlashPhase::Failed,
                            })
                            .await;
                        FlashResult {
                            device: tgt,
                            success: false,
                            bytes_written: 0,
                            error: Some(e.to_string()),
                        }
                    }
                }
            });
            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(FlashResult {
                    device: "unknown".into(),
                    success: false,
                    bytes_written: 0,
                    error: Some(format!("Task panicked: {e}")),
                }),
            }
        }

        results
    }

    async fn flash_single(
        &self,
        image_path: &Path,
        target_device: &str,
        device_index: usize,
        progress_tx: mpsc::Sender<FlashProgress>,
    ) -> anyhow::Result<()> {
        tracing::info!(
            image = %image_path.display(),
            target = target_device,
            index = device_index,
            "Starting flash"
        );

        let total_bytes = std::fs::metadata(image_path)?.len();
        let block_size = self.config.block_size;
        let verify = self.config.verify;
        let image_path_owned = image_path.to_owned();
        let target_owned = target_device.to_string();
        let device_name = target_owned.clone();

        let _ = progress_tx
            .send(FlashProgress {
                device_index,
                device_name: device_name.clone(),
                bytes_written: 0,
                total_bytes,
                phase: FlashPhase::Preparing,
            })
            .await;

        // Run synchronous write loop on a blocking thread
        let ptx = progress_tx.clone();
        let img = image_path_owned.clone();
        let tgt = target_owned.clone();
        let dname = device_name.clone();
        let didx = device_index;

        let bytes_written = tokio::task::spawn_blocking(move || -> anyhow::Result<u64> {
            let mut source = crate::io::decompress::open_image(&img)?;
            let mut target = std::fs::OpenOptions::new().write(true).open(&tgt)?;

            let mut buf = vec![0u8; block_size];
            let mut written: u64 = 0;

            let _ = ptx.blocking_send(FlashProgress {
                device_index: didx,
                device_name: dname.clone(),
                bytes_written: 0,
                total_bytes,
                phase: FlashPhase::Writing,
            });

            loop {
                let n = crate::io::read_full(&mut source, &mut buf)?;
                if n == 0 {
                    break;
                }

                target.write_all(&buf[..n])?;
                written += n as u64;

                let _ = ptx.blocking_send(FlashProgress {
                    device_index: didx,
                    device_name: dname.clone(),
                    bytes_written: written,
                    total_bytes,
                    phase: FlashPhase::Writing,
                });
            }

            target.flush()?;
            target.sync_all()?;

            Ok(written)
        })
        .await??;

        tracing::info!(bytes_written, target = target_device, "Flash write complete");

        // Verification pass
        if verify {
            let _ = progress_tx
                .send(FlashProgress {
                    device_index,
                    device_name: device_name.clone(),
                    bytes_written: 0,
                    total_bytes: bytes_written,
                    phase: FlashPhase::Verifying,
                })
                .await;

            let matches = crate::core::verify::verify_device_matches_image(
                &target_owned,
                &image_path_owned,
                block_size,
            )
            .await?;

            if !matches {
                anyhow::bail!(
                    "Verification failed for {}: device contents do not match image",
                    target_device
                );
            }

            tracing::info!(target = target_device, "Verification passed");
        }

        let _ = progress_tx
            .send(FlashProgress {
                device_index,
                device_name,
                bytes_written,
                total_bytes: bytes_written,
                phase: FlashPhase::Done,
            })
            .await;

        Ok(())
    }
}


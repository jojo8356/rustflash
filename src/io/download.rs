use std::path::Path;

use futures::StreamExt;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
/// Structure publique `DownloadProgress`
pub struct DownloadProgress {
    /// Champ public `bytes_downloaded` de la structure correspondante.
    pub bytes_downloaded: u64,
    /// Champ public `total_bytes` de la structure correspondante.
    pub total_bytes: Option<u64>,
    /// Champ public `speed_bytes_per_sec` de la structure correspondante.
    pub speed_bytes_per_sec: f64,
    /// Champ public `resumed` de la structure correspondante.
    pub resumed: bool,
}

/// Download an image from a URL with resume support.
/// If a partial file exists at `output_path`, attempts to resume using HTTP Range.
pub async fn download_image(
    url: &str,
    output_path: &Path,
    progress_tx: mpsc::Sender<DownloadProgress>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    // Check for existing partial file
    let existing_size = tokio::fs::metadata(output_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    let (response, mut file, initial_offset, resumed) = if existing_size > 0 {
        // Try resume with Range header
        tracing::info!(
            url,
            existing_bytes = existing_size,
            "Attempting resume download"
        );

        let resp = client
            .get(url)
            .header("Range", format!("bytes={existing_size}-"))
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::PARTIAL_CONTENT {
            // Server supports resume
            let f = OpenOptions::new().append(true).open(output_path).await?;
            tracing::info!(resumed_at = existing_size, "Resuming download");
            (resp, f, existing_size, true)
        } else if resp.status().is_success() {
            // Server doesn't support Range — restart from scratch
            tracing::warn!("Server does not support Range requests, restarting download");
            let resp = resp; // already have the full response
            let f = File::create(output_path).await?;
            (resp, f, 0u64, false)
        } else {
            resp.error_for_status()?;
            unreachable!()
        }
    } else {
        tracing::info!(url, output = %output_path.display(), "Starting download");
        let resp = client.get(url).send().await?.error_for_status()?;
        let f = File::create(output_path).await?;
        (resp, f, 0u64, false)
    };

    let content_length = response.content_length();
    let total_bytes = content_length.map(|cl| cl + initial_offset);

    let mut stream = response.bytes_stream();
    let mut bytes_downloaded: u64 = initial_offset;
    let start = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        bytes_downloaded += chunk.len() as u64;

        let elapsed = start.elapsed().as_secs_f64();
        let new_bytes = bytes_downloaded - initial_offset;
        let speed = if elapsed > 0.0 {
            new_bytes as f64 / elapsed
        } else {
            0.0
        };

        let _ = progress_tx
            .send(DownloadProgress {
                bytes_downloaded,
                total_bytes,
                speed_bytes_per_sec: speed,
                resumed,
            })
            .await;
    }

    file.flush().await?;
    tracing::info!(bytes = bytes_downloaded, resumed, "Download complete");

    Ok(())
}

/// Fonction publique `is_url`
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

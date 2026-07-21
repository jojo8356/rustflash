use std::collections::HashMap;
use std::path::PathBuf;

use clap::Args;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc;

use crate::core::flasher::{FlashConfig, FlashPhase, FlashProgress, Flasher};
use crate::io::download;

#[derive(Args)]
/// Structure publique `FlashArgs`
pub struct FlashArgs {
    /// Path or URL to the image file
    #[arg(short, long)]
    pub image: String,

    /// Target device(s) — can be specified multiple times for parallel flash
    #[arg(short, long, num_args = 1..)]
    pub target: Vec<String>,

    /// Verify after writing
    #[arg(long, default_value_t = true)]
    pub verify: bool,

    /// Block size in bytes (default: 4 MiB)
    #[arg(long, default_value_t = 4 * 1024 * 1024)]
    pub block_size: usize,

    /// Skip confirmation prompt
    #[arg(long, default_value_t = false)]
    pub yes: bool,

    /// Expected checksum (sha256:HASH, sha512:HASH, md5:HASH)
    #[arg(long)]
    pub checksum: Option<String>,
}

/// Fonction publique `execute`
pub async fn execute(args: &FlashArgs) -> anyhow::Result<()> {
    if args.target.is_empty() {
        anyhow::bail!("At least one --target is required");
    }

    tracing::info!(image = %args.image, targets = ?args.target, "Starting flash operation");

    // 1. Resolve image source (URL or local file)
    let image_path: PathBuf = if download::is_url(&args.image) {
        download_image_with_progress(&args.image).await?
    } else {
        PathBuf::from(&args.image)
    };

    // 2. Validate image file exists
    if !image_path.exists() {
        anyhow::bail!("Image file not found: {}", image_path.display());
    }

    // 3. Verify checksum if provided
    if let Some(ref checksum) = args.checksum {
        println!("Verifying image checksum...");
        let path = image_path.clone();
        let cs = checksum.clone();
        let matches = tokio::task::spawn_blocking(move || {
            crate::core::verify::verify_file_checksum(&path, &cs)
        })
        .await??;
        if !matches {
            anyhow::bail!("Checksum verification failed for {}", image_path.display());
        }
        println!("Checksum verified OK");
    }

    // 4. Validate all target devices exist
    for target in &args.target {
        if !std::path::Path::new(target).exists() {
            anyhow::bail!("Target device not found: {target}");
        }
    }

    // 5. Confirmation prompt
    if !args.yes {
        let image_size = std::fs::metadata(&image_path)?.len();
        println!();
        println!(
            "  Image:   {} ({})",
            image_path.display(),
            bytesize::ByteSize(image_size)
        );
        for (i, target) in args.target.iter().enumerate() {
            println!("  Target {}: {}", i + 1, target);
        }
        println!();
        if args.target.len() > 1 {
            println!(
                "WARNING: All data on {} devices will be destroyed!",
                args.target.len()
            );
        } else {
            println!("WARNING: All data on {} will be destroyed!", args.target[0]);
        }
        print!("Continue? [y/N] ");

        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // 6. Unmount all target partitions
    println!("Unmounting target partitions...");
    for target in &args.target {
        crate::device::mount::ensure_unmounted(target).await?;
    }

    // 7. Flash
    let config = FlashConfig {
        block_size: args.block_size,
        verify: args.verify,
        auto_unmount: true,
    };

    let flasher = Flasher::new(config);
    let (progress_tx, mut progress_rx) = mpsc::channel(128);

    if args.target.len() == 1 {
        // Single target — simple progress bar
        let image_clone = image_path.clone();
        let target_clone = args.target[0].clone();

        let flash_handle = tokio::spawn(async move {
            flasher
                .flash(&image_clone, &target_clone, progress_tx)
                .await
        });

        let bar = ProgressBar::new(0);
        bar.set_style(progress_style());

        while let Some(p) = progress_rx.recv().await {
            update_single_bar(&bar, &p);
        }

        flash_handle.await??;
    } else {
        // Multi-target — parallel flash with multi-progress
        let multi = MultiProgress::new();
        let mut bars: HashMap<usize, ProgressBar> = HashMap::new();

        for (i, target) in args.target.iter().enumerate() {
            let bar = multi.add(ProgressBar::new(0));
            bar.set_style(progress_style_with_prefix());
            bar.set_prefix(short_device_name(target));
            bars.insert(i, bar);
        }

        let image_clone = image_path.clone();
        let targets = args.target.clone();

        let flash_handle = tokio::spawn(async move {
            flasher
                .flash_multi(&image_clone, &targets, progress_tx)
                .await
        });

        while let Some(p) = progress_rx.recv().await {
            if let Some(bar) = bars.get(&p.device_index) {
                update_multi_bar(bar, &p);
            }
        }

        let results = flash_handle.await?;

        // Summary
        println!();
        let mut had_error = false;
        for r in &results {
            if r.success {
                println!("  {} — OK", r.device);
            } else {
                println!(
                    "  {} — FAILED: {}",
                    r.device,
                    r.error.as_deref().unwrap_or("unknown error")
                );
                had_error = true;
            }
        }

        if had_error {
            anyhow::bail!("One or more devices failed to flash");
        }
    }

    println!();
    println!("Flash complete!");

    Ok(())
}

fn progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(
            "{spinner:.green} [{elapsed}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("=> ")
}

fn progress_style_with_prefix() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(
            "{prefix:>12} {spinner:.green} [{bar:30.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("=> ")
}

fn short_device_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn update_single_bar(bar: &ProgressBar, p: &FlashProgress) {
    match p.phase {
        FlashPhase::Preparing => bar.set_message("Preparing..."),
        FlashPhase::Writing => {
            if bar.length() != Some(p.total_bytes) {
                bar.set_length(p.total_bytes);
            }
            bar.set_position(p.bytes_written);
            bar.set_message("Writing");
        }
        FlashPhase::Verifying => {
            bar.set_position(0);
            bar.set_message("Verifying...");
        }
        FlashPhase::Done => bar.finish_with_message("Done!"),
        FlashPhase::Failed => bar.abandon_with_message("FAILED"),
    }
}

fn update_multi_bar(bar: &ProgressBar, p: &FlashProgress) {
    match p.phase {
        FlashPhase::Preparing => bar.set_message("Preparing..."),
        FlashPhase::Writing => {
            if bar.length() != Some(p.total_bytes) {
                bar.set_length(p.total_bytes);
            }
            bar.set_position(p.bytes_written);
            bar.set_message("Writing");
        }
        FlashPhase::Verifying => {
            bar.set_position(0);
            bar.set_message("Verifying...");
        }
        FlashPhase::Done => bar.finish_with_message("OK"),
        FlashPhase::Failed => bar.abandon_with_message("FAILED"),
    }
}

async fn download_image_with_progress(url: &str) -> anyhow::Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let filename = url.split('/').next_back().unwrap_or("image.img");
    let download_path = temp_dir.join(filename);

    println!("Downloading image to {}...", download_path.display());

    let (dl_tx, mut dl_rx) = mpsc::channel(32);
    let path_clone = download_path.clone();
    let url_owned = url.to_string();

    let dl_handle =
        tokio::spawn(async move { download::download_image(&url_owned, &path_clone, dl_tx).await });

    let dl_bar = ProgressBar::new_spinner();
    dl_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed}] {msg}")
            .unwrap(),
    );

    while let Some(progress) = dl_rx.recv().await {
        let resumed = if progress.resumed { " (resumed)" } else { "" };
        let msg = if let Some(total) = progress.total_bytes {
            format!(
                "Downloaded {}/{} ({:.1} MB/s){}",
                bytesize::ByteSize(progress.bytes_downloaded),
                bytesize::ByteSize(total),
                progress.speed_bytes_per_sec / 1_000_000.0,
                resumed,
            )
        } else {
            format!(
                "Downloaded {} ({:.1} MB/s){}",
                bytesize::ByteSize(progress.bytes_downloaded),
                progress.speed_bytes_per_sec / 1_000_000.0,
                resumed,
            )
        };
        dl_bar.set_message(msg);
    }

    dl_handle.await??;
    dl_bar.finish_with_message("Download complete");

    Ok(download_path)
}

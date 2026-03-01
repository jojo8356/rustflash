use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::mpsc;

use crate::core::cloner::{CloneConfig, CloneMode, ClonePhase, Cloner, CompressionType};

#[derive(Args)]
pub struct CloneArgs {
    /// Source device (e.g., /dev/sda)
    #[arg(short, long)]
    pub source: String,

    /// Destination device or file path
    #[arg(short, long)]
    pub dest: String,

    /// Use smart cloning (only copy used sectors)
    #[arg(long, default_value_t = false)]
    pub smart: bool,

    /// Compress output (when cloning to file): gzip, xz, zstd
    #[arg(long)]
    pub compress: Option<String>,

    /// Verify after cloning
    #[arg(long, default_value_t = true)]
    pub verify: bool,

    /// Skip confirmation prompt
    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

pub async fn execute(args: &CloneArgs) -> anyhow::Result<()> {
    tracing::info!(source = %args.source, dest = %args.dest, "Starting clone operation");

    // Validate source exists
    if !std::path::Path::new(&args.source).exists() {
        anyhow::bail!("Source not found: {}", args.source);
    }

    // Parse compression
    let compression = match args.compress.as_deref() {
        None => None,
        Some("gzip") | Some("gz") => Some(CompressionType::Gzip),
        Some("xz") => Some(CompressionType::Xz),
        Some("zstd") | Some("zst") => Some(CompressionType::Zstd),
        Some(other) => anyhow::bail!("Unknown compression: {other}. Use gzip, xz, or zstd"),
    };

    // Confirmation prompt
    if !args.yes {
        let source_size = std::fs::metadata(&args.source)
            .map(|m| m.len())
            .unwrap_or(0);
        println!();
        println!(
            "  Source: {} ({})",
            args.source,
            bytesize::ByteSize(source_size)
        );
        println!("  Dest:   {}", args.dest);
        if let Some(ref c) = args.compress {
            println!("  Compression: {c}");
        }
        println!();
        println!("WARNING: All data on {} will be overwritten!", args.dest);
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

    // Unmount target if it's a block device
    if std::path::Path::new(&args.dest).exists() {
        let _ = crate::device::mount::ensure_unmounted(&args.dest).await;
    }

    let config = CloneConfig {
        mode: if args.smart {
            CloneMode::Smart
        } else {
            CloneMode::Raw
        },
        block_size: 4 * 1024 * 1024,
        verify: args.verify && compression.is_none(),
        compression,
    };

    let cloner = Cloner::new(config);
    let (tx, mut rx) = mpsc::channel(128);

    let source = args.source.clone();
    let dest = args.dest.clone();

    let handle = tokio::spawn(async move { cloner.clone_device(&source, &dest, tx).await });

    let bar = ProgressBar::new(0);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    while let Some(p) = rx.recv().await {
        match p.phase {
            ClonePhase::Analyzing => bar.set_message("Analyzing..."),
            ClonePhase::Copying => {
                if bar.length() != Some(p.total_bytes) {
                    bar.set_length(p.total_bytes);
                }
                bar.set_position(p.bytes_copied);
                bar.set_message("Copying");
            }
            ClonePhase::Verifying => {
                bar.set_position(0);
                bar.set_message("Verifying...");
            }
            ClonePhase::Done => bar.finish_with_message("Clone complete!"),
            ClonePhase::Failed => bar.abandon_with_message("FAILED"),
        }
    }

    handle.await??;

    println!();
    println!("Clone complete!");

    Ok(())
}

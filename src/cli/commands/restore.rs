use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::mpsc;

use crate::core::backup::{BackupConfig, BackupEngine, BackupPhase};

#[derive(Args)]
/// Structure publique `RestoreArgs`
pub struct RestoreArgs {
    /// Backup file to restore (.rfb)
    #[arg(short, long)]
    pub input: String,

    /// Target device
    #[arg(short, long)]
    pub target: String,

    /// Verify after restore
    #[arg(long, default_value_t = true)]
    pub verify: bool,

    /// Dry run — check compatibility without writing
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Skip confirmation prompt
    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

/// Fonction publique `execute`
pub async fn execute(args: &RestoreArgs) -> anyhow::Result<()> {
    tracing::info!(input = %args.input, target = %args.target, "Starting restore operation");

    // Validate input file exists
    if !std::path::Path::new(&args.input).exists() {
        anyhow::bail!("Backup file not found: {}", args.input);
    }

    // Read and display header
    let header = BackupEngine::read_header(&args.input)?;
    println!();
    println!("  Backup file:   {}", args.input);
    println!(
        "  Source device:  {}",
        header.source_device.as_deref().unwrap_or("unknown")
    );
    println!("  Created:       {}", header.created);
    println!(
        "  Source size:    {}",
        bytesize::ByteSize(header.source_size)
    );
    println!("  Compression:   {}", header.compression);
    println!(
        "  Block size:    {}",
        bytesize::ByteSize(header.block_size as u64)
    );
    println!(
        "  Checksum:      {}:{}",
        header.hash_algorithm,
        &header.checksum[..16.min(header.checksum.len())]
    );

    if args.dry_run {
        println!();
        println!("Dry run complete — backup file is valid.");
        return Ok(());
    }

    // Validate target
    if !std::path::Path::new(&args.target).exists() {
        anyhow::bail!("Target device not found: {}", args.target);
    }

    // Confirmation prompt
    if !args.yes {
        println!();
        println!("  Target: {}", args.target);
        println!();
        println!("WARNING: All data on {} will be destroyed!", args.target);
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

    // Unmount target
    crate::device::mount::ensure_unmounted(&args.target).await?;

    let config = BackupConfig::default();
    let engine = BackupEngine::new(config);
    let (tx, mut rx) = mpsc::channel(128);

    let input = args.input.clone();
    let target = args.target.clone();

    let handle = tokio::spawn(async move { engine.restore_backup(&input, &target, tx).await });

    let bar = ProgressBar::new(0);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    while let Some(p) = rx.recv().await {
        match p.phase {
            BackupPhase::Analyzing => bar.set_message("Analyzing..."),
            BackupPhase::Reading => {
                if bar.length() != Some(p.total_bytes) {
                    bar.set_length(p.total_bytes);
                }
                bar.set_position(p.bytes_processed);
                bar.set_message("Restoring");
            }
            BackupPhase::Compressing => {
                bar.set_position(p.bytes_processed);
                bar.set_message("Decompressing");
            }
            BackupPhase::Done => bar.finish_with_message("Restore complete!"),
            BackupPhase::Failed => bar.abandon_with_message("FAILED"),
        }
    }

    handle.await??;

    println!();
    println!("Restore complete!");

    Ok(())
}

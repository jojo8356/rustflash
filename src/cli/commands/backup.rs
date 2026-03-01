use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::mpsc;

use crate::core::backup::{BackupConfig, BackupEngine, BackupPhase};

#[derive(Args)]
pub struct BackupArgs {
    /// Source device to backup
    #[arg(short, long)]
    pub source: String,

    /// Output file path
    #[arg(short, long)]
    pub output: String,

    /// Compression algorithm: zstd (default), gzip, xz
    #[arg(long, default_value = "zstd")]
    pub compress: String,

    /// Compression level (depends on algorithm)
    #[arg(long)]
    pub level: Option<u32>,

    /// Only backup used sectors (smart backup)
    #[arg(long, default_value_t = false)]
    pub smart: bool,

    /// Skip confirmation prompt
    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

pub async fn execute(args: &BackupArgs) -> anyhow::Result<()> {
    tracing::info!(source = %args.source, output = %args.output, "Starting backup operation");

    // Validate source exists
    if !std::path::Path::new(&args.source).exists() {
        anyhow::bail!("Source not found: {}", args.source);
    }

    // Validate compression string
    match args.compress.as_str() {
        "gzip" | "gz" | "xz" | "zstd" | "zst" => {}
        other => anyhow::bail!("Unknown compression: {other}. Use gzip, xz, or zstd"),
    }

    // Default compression level
    let level = args.level.unwrap_or(match args.compress.as_str() {
        "gzip" | "gz" => 6,
        "xz" => 6,
        _ => 3, // zstd
    });

    // Confirmation prompt
    if !args.yes {
        let source_size = std::fs::metadata(&args.source)
            .map(|m| m.len())
            .unwrap_or(0);
        println!();
        println!(
            "  Source:      {} ({})",
            args.source,
            bytesize::ByteSize(source_size)
        );
        println!("  Output:      {}", args.output);
        println!("  Compression: {} (level {})", args.compress, level);
        println!();
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

    let config = BackupConfig {
        block_size: 4 * 1024 * 1024,
        compression: args.compress.clone(),
        compression_level: level,
        smart: args.smart,
    };

    let engine = BackupEngine::new(config);
    let (tx, mut rx) = mpsc::channel(128);

    let source = args.source.clone();
    let output = args.output.clone();

    let handle =
        tokio::spawn(async move { engine.create_backup(&source, &output, tx).await });

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
                bar.set_message("Reading");
            }
            BackupPhase::Compressing => {
                if bar.length() != Some(p.total_bytes) {
                    bar.set_length(p.total_bytes);
                }
                bar.set_position(p.bytes_processed);
                bar.set_message("Compressing");
            }
            BackupPhase::Done => bar.finish_with_message("Backup complete!"),
            BackupPhase::Failed => bar.abandon_with_message("FAILED"),
        }
    }

    handle.await??;

    // Show output file size
    if let Ok(meta) = std::fs::metadata(&args.output) {
        println!();
        println!(
            "Backup saved to {} ({})",
            args.output,
            bytesize::ByteSize(meta.len())
        );
    }

    Ok(())
}

use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::mpsc;

use crate::core::partition::{EraseMethod, FsType, PartitionManager, TableType, parse_size};

#[derive(Args)]
/// Structure publique `PartitionArgs`
pub struct PartitionArgs {
    /// Target device
    pub device: String,

    #[command(subcommand)]
    /// Champ public `action` de la structure correspondante.
    pub action: PartitionAction,
}

#[derive(Subcommand)]
/// Énumération publique `PartitionAction`
pub enum PartitionAction {
    /// Create a new partition table (gpt or mbr)
    Create {
        /// Table type: gpt or mbr
        #[arg(value_parser = ["gpt", "mbr"])]
        table_type: String,
    },

    /// Add a new partition
    Add {
        /// Filesystem type: ext4, fat32, ntfs, exfat, swap
        #[arg(short = 't', long)]
        fs_type: String,

        /// Size (e.g., 256M, 4G, remaining)
        #[arg(short, long)]
        size: String,

        /// Partition label
        #[arg(short, long)]
        label: Option<String>,

        /// Partition flags (comma-separated): boot, esp, lvm, raid
        #[arg(short, long)]
        flag: Option<String>,
    },

    /// Delete a partition
    Delete {
        /// Partition number
        #[arg(short, long)]
        number: u32,
    },

    /// Format a partition
    Format {
        /// Partition number
        #[arg(short, long)]
        number: u32,

        /// Filesystem type
        #[arg(short = 't', long)]
        fs_type: String,

        /// Volume label
        #[arg(short, long)]
        label: Option<String>,
    },

    /// Show partition table
    Show,

    /// Secure erase: zero, random, dod
    Erase {
        /// Erase method: zero, random, dod
        #[arg(short, long, default_value = "zero")]
        method: String,
    },
}

/// Fonction publique `execute`
pub async fn execute(args: &PartitionArgs) -> anyhow::Result<()> {
    tracing::info!(device = %args.device, "Starting partition operation");

    if !std::path::Path::new(&args.device).exists() {
        anyhow::bail!("Device not found: {}", args.device);
    }

    match &args.action {
        PartitionAction::Show => cmd_show(&args.device),

        PartitionAction::Create { table_type } => {
            let tt = match table_type.as_str() {
                "gpt" => TableType::Gpt,
                "mbr" => TableType::Mbr,
                _ => anyhow::bail!("Invalid table type: {table_type}"),
            };
            println!(
                "Creating {} partition table on {}...",
                table_type.to_uppercase(),
                args.device
            );
            PartitionManager::create_table(&args.device, tt)?;
            println!("Partition table created.");
            Ok(())
        }

        PartitionAction::Add {
            fs_type,
            size,
            label,
            flag,
        } => {
            let fs = FsType::parse(fs_type);
            let size_bytes = parse_size(size)?;
            let flags: Vec<&str> = flag
                .as_deref()
                .map(|f| f.split(',').collect())
                .unwrap_or_default();

            let size_display = if size_bytes == 0 {
                "remaining".to_string()
            } else {
                bytesize::ByteSize(size_bytes).to_string()
            };
            println!(
                "Adding {} partition ({}) to {}...",
                fs_type, size_display, args.device
            );

            PartitionManager::add_partition(
                &args.device,
                fs,
                size_bytes,
                label.as_deref(),
                &flags,
            )?;
            println!("Partition added.");
            Ok(())
        }

        PartitionAction::Delete { number } => {
            println!("Deleting partition {} on {}...", number, args.device);

            print!("Continue? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }

            PartitionManager::delete_partition(&args.device, *number)?;
            println!("Partition deleted.");
            Ok(())
        }

        PartitionAction::Format {
            number,
            fs_type,
            label,
        } => {
            let fs = FsType::parse(fs_type);
            println!(
                "Formatting partition {} as {} on {}...",
                number, fs_type, args.device
            );

            println!("WARNING: All data on the partition will be lost!");
            print!("Continue? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }

            PartitionManager::format_partition(&args.device, *number, fs, label.as_deref())?;
            println!("Partition formatted.");
            Ok(())
        }

        PartitionAction::Erase { method } => {
            let erase_method = match method.as_str() {
                "zero" => EraseMethod::Zero,
                "random" => EraseMethod::Random,
                "dod" => EraseMethod::Dod,
                _ => anyhow::bail!("Unknown erase method: {method}. Use zero, random, or dod"),
            };

            println!(
                "WARNING: This will irreversibly destroy ALL data on {}!",
                args.device
            );
            println!("Erase method: {method}");
            print!("Type YES to confirm: ");
            use std::io::Write;
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() != "YES" {
                println!("Aborted.");
                return Ok(());
            }

            let (tx, mut rx) = mpsc::channel(64);

            let device = args.device.clone();
            let handle = tokio::spawn(async move {
                PartitionManager::secure_erase(&device, erase_method, Some(tx)).await
            });

            let bar = ProgressBar::new(0);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed}] [{bar:40.red/darkgray}] {bytes}/{total_bytes} ({bytes_per_sec}) Pass {msg}")
                    .unwrap()
                    .progress_chars("=> "),
            );

            while let Some(p) = rx.recv().await {
                if bar.length() != Some(p.total_bytes) {
                    bar.set_length(p.total_bytes);
                }
                bar.set_position(p.bytes_erased);
                bar.set_message(format!("{}/{}", p.pass, p.total_passes));
            }

            handle.await??;
            bar.finish_with_message("Erase complete!");
            println!();
            println!("Secure erase complete.");
            Ok(())
        }
    }
}

fn cmd_show(device: &str) -> anyhow::Result<()> {
    let (table_type, partitions) = PartitionManager::read_table(device)?;

    let type_str = match table_type {
        TableType::Gpt => "GPT",
        TableType::Mbr => "MBR",
    };

    println!();
    println!("  Device: {device}");
    println!("  Table:  {type_str}");
    println!();

    if partitions.is_empty() {
        println!("  No partitions found.");
    } else {
        println!(
            "  {:>3}  {:>12}  {:>12}  {:>10}  {:>8}  Label",
            "#", "Start", "End", "Size", "Type"
        );
        println!("  {}", "-".repeat(70));

        for p in &partitions {
            let label = p.label.as_deref().unwrap_or("");
            println!(
                "  {:>3}  {:>12}  {:>12}  {:>10}  {:>8}  {}",
                p.number,
                p.start_sector,
                p.end_sector,
                bytesize::ByteSize(p.size_bytes),
                p.fs_type.as_str(),
                label,
            );
        }
    }

    println!();
    Ok(())
}

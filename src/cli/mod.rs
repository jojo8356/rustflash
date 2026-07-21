/// Module public `commands`
pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rustflash",
    version,
    about = "Fast, cross-platform tool for flashing images, cloning disks, and managing partitions"
)]
/// Structure publique `Cli`
pub struct Cli {
    #[command(subcommand)]
    /// Champ public `command` de la structure correspondante.
    pub command: Option<Commands>,

    /// Run in TUI mode (default when no command is given)
    #[arg(long, default_value_t = false)]
    pub tui: bool,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
/// Énumération publique `Commands`
pub enum Commands {
    /// Flash an image to a target device
    Flash(commands::flash::FlashArgs),

    /// Clone a disk to another disk or image file
    Clone(commands::clone::CloneArgs),

    /// Create a backup of a device
    Backup(commands::backup::BackupArgs),

    /// Restore a backup to a device
    Restore(commands::restore::RestoreArgs),

    /// Manage partitions on a device
    Partition(commands::partition::PartitionArgs),

    /// List available devices
    List(commands::list::ListArgs),
}

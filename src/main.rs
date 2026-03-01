use clap::Parser;
use tracing_subscriber::EnvFilter;

use rustflash::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .init();

    // Load config
    let _config = rustflash::config::AppConfig::load().unwrap_or_default();

    match cli.command {
        Some(Commands::Flash(ref args)) => rustflash::cli::commands::flash::execute(args).await,
        Some(Commands::Clone(ref args)) => rustflash::cli::commands::clone::execute(args).await,
        Some(Commands::Backup(ref args)) => rustflash::cli::commands::backup::execute(args).await,
        Some(Commands::Restore(ref args)) => {
            rustflash::cli::commands::restore::execute(args).await
        }
        Some(Commands::Partition(ref args)) => {
            rustflash::cli::commands::partition::execute(args).await
        }
        Some(Commands::List(ref args)) => rustflash::cli::commands::list::execute(args).await,
        None => rustflash::tui::run().await,
    }
}

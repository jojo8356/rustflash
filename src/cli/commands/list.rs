use clap::Args;

#[derive(Args)]
/// Structure publique `ListArgs`
pub struct ListArgs {
    /// Output as JSON
    #[arg(long, default_value_t = false)]
    pub json: bool,

    /// Show all devices including system disks
    #[arg(long, default_value_t = false)]
    pub all: bool,
}

/// Fonction publique `execute`
pub async fn execute(args: &ListArgs) -> anyhow::Result<()> {
    let devices = crate::device::detect::list_devices(args.all).await?;

    if args.json {
        let json = serde_json::to_string_pretty(&devices)?;
        println!("{json}");
    } else {
        if devices.is_empty() {
            println!("No removable devices found.");
            return Ok(());
        }

        println!("{:<15} {:<12} {:<20} MOUNTPOINT", "DEVICE", "SIZE", "MODEL");
        println!("{}", "-".repeat(60));
        for dev in &devices {
            println!(
                "{:<15} {:<12} {:<20} {}",
                dev.path,
                bytesize::ByteSize(dev.size).to_string(),
                dev.model.as_deref().unwrap_or("-"),
                dev.mount_point.as_deref().unwrap_or("-"),
            );
        }
    }

    Ok(())
}

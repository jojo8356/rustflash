/// Fonction publique `unmount_all_partitions`
pub async fn unmount_all_partitions(device_path: &str) -> anyhow::Result<()> {
    let enumerator = crate::platform::get_enumerator();
    enumerator.unmount_device(device_path)?;
    tracing::info!(device = device_path, "All partitions unmounted");
    Ok(())
}

/// Fonction publique `ensure_unmounted`
pub async fn ensure_unmounted(device_path: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let mounts = std::fs::read_to_string("/proc/mounts")?;
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0].starts_with(device_path) {
                tracing::info!(partition = parts[0], mount = parts[1], "Unmounting partition");
                let output = std::process::Command::new("umount")
                    .arg(parts[0])
                    .output()?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.contains("not mounted") {
                        anyhow::bail!("Failed to unmount {}: {}", parts[0], stderr);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        unmount_all_partitions(device_path).await?;
    }

    Ok(())
}

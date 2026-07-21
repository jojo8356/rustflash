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
                tracing::info!(
                    partition = parts[0],
                    mount = parts[1],
                    "Unmounting partition"
                );
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
        if !is_block_device_path(device_path) {
            tracing::info!(device = device_path, "Skipping unmount for non-device path");
            return Ok(());
        }

        unmount_all_partitions(device_path).await?;
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn is_block_device_path(device_path: &str) -> bool {
    if let Ok(_metadata) = std::fs::metadata(device_path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let ft = metadata.file_type();
            if ft.is_block_device() || ft.is_char_device() {
                return true;
            }
        }
    }

    let normalized = device_path.to_ascii_lowercase();
    #[cfg(unix)]
    {
        if normalized.starts_with("/dev/") {
            return true;
        }
    }
    #[cfg(windows)]
    {
        if normalized.starts_with(r"\\.\") {
            return true;
        }
    }

    false
}

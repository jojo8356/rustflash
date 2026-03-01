use crate::device::detect::DeviceInfo;
use crate::platform::DeviceEnumerator;

pub struct LinuxEnumerator;

impl DeviceEnumerator for LinuxEnumerator {
    fn list_devices(&self, include_system: bool) -> anyhow::Result<Vec<DeviceInfo>> {
        let mut devices = Vec::new();

        // Read from /sys/block/ to enumerate block devices
        let block_dir = std::fs::read_dir("/sys/block")?;

        for entry in block_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip virtual devices (loop, ram, dm-, etc.)
            if name.starts_with("loop")
                || name.starts_with("ram")
                || name.starts_with("dm-")
                || name.starts_with("sr")
                || name.starts_with("zram")
            {
                continue;
            }

            let device_path = format!("/dev/{name}");
            let sys_path = format!("/sys/block/{name}");

            // Check if removable
            let removable = std::fs::read_to_string(format!("{sys_path}/removable"))
                .unwrap_or_default()
                .trim()
                == "1";

            if !include_system && !removable && self.is_system_disk(&device_path) {
                continue;
            }

            // Read size (in 512-byte sectors)
            let size_sectors: u64 = std::fs::read_to_string(format!("{sys_path}/size"))
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0);
            let size = size_sectors * 512;

            // Read model
            let model = std::fs::read_to_string(format!("{sys_path}/device/model"))
                .ok()
                .map(|s| s.trim().to_string());

            // Check mount points
            let mount_point = find_mount_point(&device_path);

            devices.push(DeviceInfo {
                path: device_path,
                size,
                model,
                removable,
                mount_point,
            });
        }

        Ok(devices)
    }

    fn unmount_device(&self, device_path: &str) -> anyhow::Result<()> {
        tracing::info!(device = device_path, "Unmounting device");

        // Find all partitions of this device and unmount them
        let output = std::process::Command::new("umount")
            .arg(device_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("not mounted") {
                anyhow::bail!("Failed to unmount {device_path}: {stderr}");
            }
        }

        Ok(())
    }

    fn is_system_disk(&self, device_path: &str) -> bool {
        // Check if any partition of this device is mounted as /, /home, /boot, etc.
        let Ok(mounts) = std::fs::read_to_string("/proc/mounts") else {
            return false;
        };

        let system_mounts = ["/", "/home", "/boot", "/boot/efi", "/var"];

        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[0].starts_with(device_path) {
                if system_mounts.contains(&parts[1]) {
                    return true;
                }
            }
        }

        false
    }
}

fn find_mount_point(device_path: &str) -> Option<String> {
    let mounts = std::fs::read_to_string("/proc/mounts").ok()?;
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0].starts_with(device_path) {
            return Some(parts[1].to_string());
        }
    }
    None
}

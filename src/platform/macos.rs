use crate::device::detect::DeviceInfo;
use crate::platform::DeviceEnumerator;
use std::process::Command;

/// Structure publique `MacosEnumerator`
pub struct MacosEnumerator;

impl DeviceEnumerator for MacosEnumerator {
    fn list_devices(&self, include_system: bool) -> anyhow::Result<Vec<DeviceInfo>> {
        let output = Command::new("diskutil").arg("list").output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("diskutil list failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        for raw_line in stdout.lines() {
            let line = raw_line.trim();
            if !line.starts_with("/dev/disk") {
                continue;
            }

            let Some((device_path, metadata)) = line.split_once(' ') else {
                continue;
            };
            let device_path = device_path.trim().trim_end_matches(':').to_string();
            if device_path.contains("disk") {
                // Keep only whole-disk entries (skip partitions like /dev/disk2s1).
                if device_path.contains("s") {
                    continue;
                }
            }

            let metadata_lower = metadata.to_ascii_lowercase();
            let removable = metadata_lower.contains("external");
            if !include_system && is_internal_system_disk(&device_path) && !removable {
                continue;
            }

            let model = read_macos_disk_info(&device_path, |line| {
                line.starts_with("Device / Media Name:") || line.starts_with("Device Name:")
            })
            .and_then(|line| {
                line.split_once(':')
                    .map(|(_, value)| value.trim().to_string())
            });

            let size = read_macos_disk_info(&device_path, |line| {
                line.starts_with("Total Size:") || line.starts_with("Disk Size:")
            })
            .and_then(parse_bytes_from_diskutil_line)
            .unwrap_or(0);

            let mount_point =
                read_macos_disk_info(&device_path, |line| line.starts_with("Mount Point:"))
                    .and_then(|line| {
                        let value = line
                            .split_once(':')
                            .map(|(_, value)| value.trim())
                            .unwrap_or("");
                        if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    });

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
        // Use diskutil unmountDisk
        let output = Command::new("diskutil")
            .args(["unmountDisk", device_path])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to unmount {device_path}: {stderr}");
        }

        Ok(())
    }

    fn is_system_disk(&self, device_path: &str) -> bool {
        is_internal_system_disk(device_path)
    }
}

fn is_internal_system_disk(device_path: &str) -> bool {
    let normalized = device_path.to_ascii_lowercase();
    normalized == "/dev/disk0"
        || normalized == "disk0"
        || normalized.ends_with("disk0")
        || normalized.ends_with("disk0s0")
}

fn read_macos_disk_info(device_path: &str, selector: fn(&str) -> bool) -> Option<String> {
    let output = Command::new("diskutil")
        .args(["info", device_path])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .map(|line| line.trim().to_string())
        .find(|line| selector(line))
}

fn parse_bytes_from_diskutil_line(line: String) -> Option<u64> {
    let line = line.trim();
    let open = line.find('(')?;
    let close = line.rfind(" Bytes)")?;
    if close <= open + 1 {
        return None;
    }
    let bytes_part = &line[open + 1..close];
    let bytes = bytes_part
        .trim_end_matches(" Bytes")
        .rsplit(' ')
        .next()?
        .replace(',', "");
    bytes.parse::<u64>().ok()
}

use crate::device::detect::DeviceInfo;
use crate::platform::DeviceEnumerator;

pub struct MacosEnumerator;

impl DeviceEnumerator for MacosEnumerator {
    fn list_devices(&self, _include_system: bool) -> anyhow::Result<Vec<DeviceInfo>> {
        // TODO: Use `diskutil list -plist` to enumerate disks
        // Parse plist output for device info
        // Filter out internal/system disks unless include_system is true

        tracing::warn!("macOS device enumeration not yet implemented");
        Ok(Vec::new())
    }

    fn unmount_device(&self, device_path: &str) -> anyhow::Result<()> {
        // Use diskutil unmountDisk
        let output = std::process::Command::new("diskutil")
            .args(["unmountDisk", device_path])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to unmount {device_path}: {stderr}");
        }

        Ok(())
    }

    fn is_system_disk(&self, device_path: &str) -> bool {
        // disk0 is typically the system disk on macOS
        device_path.contains("disk0")
    }
}

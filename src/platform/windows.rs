use crate::device::detect::DeviceInfo;
use crate::platform::DeviceEnumerator;

pub struct WindowsEnumerator;

impl DeviceEnumerator for WindowsEnumerator {
    fn list_devices(&self, _include_system: bool) -> anyhow::Result<Vec<DeviceInfo>> {
        // TODO: Use Windows SetupDi* API or WMI to enumerate physical drives
        // - CreateFile on \\.\PhysicalDriveN
        // - IOCTL_STORAGE_QUERY_PROPERTY for model/vendor
        // - IOCTL_DISK_GET_DRIVE_GEOMETRY_EX for size
        // - Check if drive is removable via IOCTL_STORAGE_GET_DEVICE_NUMBER

        tracing::warn!("Windows device enumeration not yet implemented");
        Ok(Vec::new())
    }

    fn unmount_device(&self, device_path: &str) -> anyhow::Result<()> {
        // TODO: Use FSCTL_LOCK_VOLUME + FSCTL_DISMOUNT_VOLUME
        tracing::warn!(device = device_path, "Windows unmount not yet implemented");
        Ok(())
    }

    fn is_system_disk(&self, _device_path: &str) -> bool {
        // TODO: Check if drive contains C:\ or system partition
        false
    }
}

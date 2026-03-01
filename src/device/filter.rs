use super::detect::DeviceInfo;

pub fn filter_removable(devices: &[DeviceInfo]) -> Vec<&DeviceInfo> {
    devices.iter().filter(|d| d.removable).collect()
}

pub fn filter_by_min_size(devices: &[DeviceInfo], min_bytes: u64) -> Vec<&DeviceInfo> {
    devices.iter().filter(|d| d.size >= min_bytes).collect()
}

pub fn is_safe_target(device: &DeviceInfo) -> bool {
    let enumerator = crate::platform::get_enumerator();

    // Must not be a system disk
    if enumerator.is_system_disk(&device.path) {
        return false;
    }

    // Prefer removable devices
    device.removable
}

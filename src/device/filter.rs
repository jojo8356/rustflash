use super::detect::DeviceInfo;

/// Fonction publique `filter_removable`
pub fn filter_removable(devices: &[DeviceInfo]) -> Vec<&DeviceInfo> {
    devices.iter().filter(|d| d.removable).collect()
}

/// Fonction publique `filter_by_min_size`
pub fn filter_by_min_size(devices: &[DeviceInfo], min_bytes: u64) -> Vec<&DeviceInfo> {
    devices.iter().filter(|d| d.size >= min_bytes).collect()
}

/// Fonction publique `is_safe_target`
pub fn is_safe_target(device: &DeviceInfo) -> bool {
    let enumerator = crate::platform::get_enumerator();

    // Must not be a system disk
    if enumerator.is_system_disk(&device.path) {
        return false;
    }

    // Prefer removable devices
    device.removable
}

#[cfg(test)]
mod tests {
    use super::filter_by_min_size;
    use super::filter_removable;
    use super::is_safe_target;
    use crate::device::detect::DeviceInfo;

    fn sample_devices() -> Vec<DeviceInfo> {
        vec![
            DeviceInfo {
                path: "sda".into(),
                size: 1_u64 * 1024 * 1024 * 1024,
                model: Some("System SSD".into()),
                removable: false,
                mount_point: None,
            },
            DeviceInfo {
                path: "sdb".into(),
                size: 64_u64 * 1024 * 1024,
                model: Some("USB Stick".into()),
                removable: true,
                mount_point: Some("/mnt/usb".into()),
            },
            DeviceInfo {
                path: "sdc".into(),
                size: 32_u64 * 1024 * 1024,
                model: None,
                removable: true,
                mount_point: None,
            },
        ]
    }

    #[test]
    fn filters_removable_devices_only() {
        let devices = sample_devices();
        let rem = filter_removable(&devices);
        assert_eq!(rem.len(), 2);
        assert!(rem.iter().all(|d| d.removable));
    }

    #[test]
    fn filters_by_min_size_inclusive() {
        let devices = sample_devices();
        let large = filter_by_min_size(&devices, 64_u64 * 1024 * 1024);
        assert_eq!(large.len(), 2);
        assert!(large.iter().all(|d| d.size >= 64_u64 * 1024 * 1024));
    }

    #[test]
    fn unsafe_when_not_removable() {
        let device = DeviceInfo {
            path: "/tmp/never-a-device".into(),
            size: 1024,
            model: Some("Generic".into()),
            removable: false,
            mount_point: None,
        };
        assert!(!is_safe_target(&device));
    }

    #[test]
    fn safe_when_removable_and_not_system() {
        let device = DeviceInfo {
            path: "/tmp/fake-removable".into(),
            size: 1024,
            model: Some("Generic".into()),
            removable: true,
            mount_point: None,
        };
        assert!(is_safe_target(&device));
    }
}

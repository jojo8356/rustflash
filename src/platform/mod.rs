/// Module public `linux`
pub mod linux;
/// Module public `macos`
pub mod macos;
/// Module public `windows`
pub mod windows;

use crate::device::detect::DeviceInfo;

/// Trait public `DeviceEnumerator`
pub trait DeviceEnumerator {
    /// Élément public `list_devices` exposé par l'API.
    fn list_devices(&self, include_system: bool) -> anyhow::Result<Vec<DeviceInfo>>;
    /// Élément public `unmount_device` exposé par l'API.
    fn unmount_device(&self, device_path: &str) -> anyhow::Result<()>;
    /// Élément public `is_system_disk` exposé par l'API.
    fn is_system_disk(&self, device_path: &str) -> bool;
}

/// Fonction publique `get_enumerator`
pub fn get_enumerator() -> Box<dyn DeviceEnumerator> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxEnumerator)
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsEnumerator)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacosEnumerator)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        compile_error!("Unsupported platform")
    }
}

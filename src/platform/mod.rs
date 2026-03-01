pub mod linux;
pub mod macos;
pub mod windows;

use crate::device::detect::DeviceInfo;

pub trait DeviceEnumerator {
    fn list_devices(&self, include_system: bool) -> anyhow::Result<Vec<DeviceInfo>>;
    fn unmount_device(&self, device_path: &str) -> anyhow::Result<()>;
    fn is_system_disk(&self, device_path: &str) -> bool;
}

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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub path: String,
    pub size: u64,
    pub model: Option<String>,
    pub removable: bool,
    pub mount_point: Option<String>,
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = bytesize::ByteSize(self.size);
        let model = self.model.as_deref().unwrap_or("Unknown");
        write!(f, "{} — {} ({})", self.path, model, size)
    }
}

pub async fn list_devices(include_system: bool) -> anyhow::Result<Vec<DeviceInfo>> {
    let enumerator = crate::platform::get_enumerator();
    enumerator.list_devices(include_system)
}

pub async fn get_device(path: &str) -> anyhow::Result<DeviceInfo> {
    let devices = list_devices(true).await?;
    devices
        .into_iter()
        .find(|d| d.path == path)
        .ok_or_else(|| anyhow::anyhow!("Device not found: {path}"))
}

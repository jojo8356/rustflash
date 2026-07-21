use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Structure publique `DeviceInfo`
pub struct DeviceInfo {
    /// Champ public `path` de la structure correspondante.
    pub path: String,
    /// Champ public `size` de la structure correspondante.
    pub size: u64,
    /// Champ public `model` de la structure correspondante.
    pub model: Option<String>,
    /// Champ public `removable` de la structure correspondante.
    pub removable: bool,
    /// Champ public `mount_point` de la structure correspondante.
    pub mount_point: Option<String>,
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = bytesize::ByteSize(self.size);
        let model = self.model.as_deref().unwrap_or("Unknown");
        write!(f, "{} — {} ({})", self.path, model, size)
    }
}

/// Fonction publique `list_devices`
pub async fn list_devices(include_system: bool) -> anyhow::Result<Vec<DeviceInfo>> {
    let enumerator = crate::platform::get_enumerator();
    enumerator.list_devices(include_system)
}

/// Fonction publique `get_device`
pub async fn get_device(path: &str) -> anyhow::Result<DeviceInfo> {
    let devices = list_devices(true).await?;
    devices
        .into_iter()
        .find(|d| d.path == path)
        .ok_or_else(|| anyhow::anyhow!("Device not found: {path}"))
}

#[cfg(test)]
mod tests {
    use super::DeviceInfo;

    #[test]
    fn displays_device_without_model_as_unknown() {
        let device = DeviceInfo {
            path: "/dev/sdx".into(),
            size: 4 * 1024 * 1024 * 1024,
            model: None,
            removable: false,
            mount_point: Some("/mnt/foo".into()),
        };

        let text = format!("{device}");
        assert!(text.contains("/dev/sdx"));
        assert!(text.contains("Unknown"));
        assert!(text.contains("4"));
    }

    #[test]
    fn displays_device_with_model() {
        let device = DeviceInfo {
            path: "/dev/sdy".into(),
            size: 128 * 1024 * 1024,
            model: Some("AcmeDrive".into()),
            removable: true,
            mount_point: None,
        };

        let text = format!("{device}");
        assert!(text.contains("AcmeDrive"));
    }
}

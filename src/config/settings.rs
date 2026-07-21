use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Structure publique `AppConfig`
pub struct AppConfig {
    #[serde(default)]
    /// Champ public `general` de la structure correspondante.
    pub general: GeneralConfig,
    #[serde(default)]
    /// Champ public `flash` de la structure correspondante.
    pub flash: FlashConfig,
    #[serde(default)]
    /// Champ public `network` de la structure correspondante.
    pub network: NetworkConfig,
    #[serde(default)]
    /// Champ public `backup` de la structure correspondante.
    pub backup: BackupConfig,
    #[serde(default)]
    /// Champ public `logging` de la structure correspondante.
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Structure publique `GeneralConfig`
pub struct GeneralConfig {
    #[serde(default = "default_theme")]
    /// Champ public `theme` de la structure correspondante.
    pub theme: String,
    #[serde(default = "default_language")]
    /// Champ public `language` de la structure correspondante.
    pub language: String,
    #[serde(default = "default_true")]
    /// Champ public `confirm_destructive` de la structure correspondante.
    pub confirm_destructive: bool,
    #[serde(default)]
    /// Champ public `show_system_drives` de la structure correspondante.
    pub show_system_drives: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Structure publique `FlashConfig`
pub struct FlashConfig {
    #[serde(default = "default_block_size")]
    /// Champ public `block_size` de la structure correspondante.
    pub block_size: usize,
    #[serde(default = "default_true")]
    /// Champ public `verify_after_write` de la structure correspondante.
    pub verify_after_write: bool,
    #[serde(default = "default_true")]
    /// Champ public `auto_unmount` de la structure correspondante.
    pub auto_unmount: bool,
    #[serde(default)]
    /// Champ public `decompress_threads` de la structure correspondante.
    pub decompress_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Structure publique `NetworkConfig`
pub struct NetworkConfig {
    #[serde(default = "default_timeout")]
    /// Champ public `download_timeout` de la structure correspondante.
    pub download_timeout: u64,
    #[serde(default = "default_true")]
    /// Champ public `resume_downloads` de la structure correspondante.
    pub resume_downloads: bool,
    #[serde(default)]
    /// Champ public `proxy` de la structure correspondante.
    pub proxy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Structure publique `BackupConfig`
pub struct BackupConfig {
    #[serde(default = "default_compression")]
    /// Champ public `default_compression` de la structure correspondante.
    pub default_compression: String,
    #[serde(default = "default_compression_level")]
    /// Champ public `compression_level` de la structure correspondante.
    pub compression_level: u32,
    #[serde(default)]
    /// Champ public `default_output_dir` de la structure correspondante.
    pub default_output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Structure publique `LoggingConfig`
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    /// Champ public `level` de la structure correspondante.
    pub level: String,
    #[serde(default)]
    /// Champ public `file` de la structure correspondante.
    pub file: String,
}

// Default value helpers
fn default_theme() -> String {
    "dark".into()
}
fn default_language() -> String {
    "en".into()
}
fn default_true() -> bool {
    true
}
fn default_block_size() -> usize {
    4 * 1024 * 1024
}
fn default_timeout() -> u64 {
    300
}
fn default_compression() -> String {
    "zstd".into()
}
fn default_compression_level() -> u32 {
    3
}
fn default_log_level() -> String {
    "info".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            flash: FlashConfig::default(),
            network: NetworkConfig::default(),
            backup: BackupConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            language: default_language(),
            confirm_destructive: true,
            show_system_drives: false,
        }
    }
}

impl Default for FlashConfig {
    fn default() -> Self {
        Self {
            block_size: default_block_size(),
            verify_after_write: true,
            auto_unmount: true,
            decompress_threads: 0,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            download_timeout: default_timeout(),
            resume_downloads: true,
            proxy: String::new(),
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            default_compression: default_compression(),
            compression_level: default_compression_level(),
            default_output_dir: String::new(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: String::new(),
        }
    }
}

impl AppConfig {
    /// Fonction publique `load`
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Fonction publique `save`
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Fonction publique `config_path`
    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("", "", "rustflash")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}

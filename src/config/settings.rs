use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub flash: FlashConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub backup: BackupConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
    #[serde(default)]
    pub show_system_drives: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashConfig {
    #[serde(default = "default_block_size")]
    pub block_size: usize,
    #[serde(default = "default_true")]
    pub verify_after_write: bool,
    #[serde(default = "default_true")]
    pub auto_unmount: bool,
    #[serde(default)]
    pub decompress_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_timeout")]
    pub download_timeout: u64,
    #[serde(default = "default_true")]
    pub resume_downloads: bool,
    #[serde(default)]
    pub proxy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    #[serde(default = "default_compression")]
    pub default_compression: String,
    #[serde(default = "default_compression_level")]
    pub compression_level: u32,
    #[serde(default)]
    pub default_output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub file: String,
}

// Default value helpers
fn default_theme() -> String { "dark".into() }
fn default_language() -> String { "en".into() }
fn default_true() -> bool { true }
fn default_block_size() -> usize { 4 * 1024 * 1024 }
fn default_timeout() -> u64 { 300 }
fn default_compression() -> String { "zstd".into() }
fn default_compression_level() -> u32 { 3 }
fn default_log_level() -> String { "info".into() }

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

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("", "", "rustflash")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}

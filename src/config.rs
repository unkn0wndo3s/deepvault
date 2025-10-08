//! Configuration management for DeepVault

use crate::{DeepVaultError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub security: SecurityConfig,
    pub ui: UiConfig,
    pub paths: PathConfig,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub default_algorithm: String,
    pub default_kdf_params: crate::crypto::KeyDerivation,
    pub enable_hidden_volumes: bool,
    pub auto_unmount_timeout: u64, // seconds
    pub secure_wipe_passes: u32,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub language: String,
    pub show_advanced_options: bool,
    pub confirm_destructive_actions: bool,
}

/// Path configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub config_dir: PathBuf,
    pub log_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            security: SecurityConfig::default(),
            ui: UiConfig::default(),
            paths: PathConfig::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            default_algorithm: "Argon2id".to_string(),
            default_kdf_params: crate::crypto::KeyDerivation::default(),
            enable_hidden_volumes: true,
            auto_unmount_timeout: 300, // 5 minutes
            secure_wipe_passes: 3,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "fr".to_string(),
            show_advanced_options: false,
            confirm_destructive_actions: true,
        }
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("deepvault"),
            log_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("deepvault")
                .join("logs"),
            temp_dir: std::env::temp_dir().join("deepvault"),
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn load(config_path: &PathBuf) -> Result<Self> {
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(config_path)
            .map_err(|e| DeepVaultError::Config(format!("Failed to read config file: {}", e)))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| DeepVaultError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, config_path: &PathBuf) -> Result<()> {
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DeepVaultError::Config(format!("Failed to create config directory: {}", e)))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| DeepVaultError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(config_path, content)
            .map_err(|e| DeepVaultError::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Get default config path
    pub fn default_config_path() -> PathBuf {
        PathConfig::default().config_dir.join("config.json")
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.security.auto_unmount_timeout == 0 {
            return Err(DeepVaultError::Config("Auto unmount timeout cannot be zero".to_string()));
        }

        if self.security.secure_wipe_passes == 0 {
            return Err(DeepVaultError::Config("Secure wipe passes cannot be zero".to_string()));
        }

        if !["Argon2id", "Argon2i", "Argon2d"].contains(&self.security.default_algorithm.as_str()) {
            return Err(DeepVaultError::Config("Invalid default algorithm".to_string()));
        }

        Ok(())
    }
}

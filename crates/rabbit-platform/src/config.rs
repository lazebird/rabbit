//! Platform-specific configuration storage
//!
//! Configuration loading follows a layered approach:
//! 1. Load from disk (TOML file)
//! 2. Merge with defaults for any missing/partial fields
//! 3. Return complete config ready for use

use super::{PlatformError, Result};
use rabbit_models::config::*;
use std::path::PathBuf;
use tracing::info;

const CONFIG_FILE: &str = "config.toml";

/// Get the configuration directory
pub fn get_config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| PlatformError::Config("Cannot find config directory".into()))?
        .join("rabbit");
    
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Get the data directory
pub fn get_data_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| PlatformError::Config("Cannot find data directory".into()))?
        .join("rabbit");
    
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Load configuration from disk with defaults merge
///
/// This is the primary entry point for loading configuration.
/// It loads from TOML file and merges with defaults for any missing fields.
pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    
    let mut config = if !config_path.exists() {
        info!("Config file not found, using defaults");
        AppConfig::default()
    } else {
        let content = std::fs::read_to_string(&config_path)?;
        let loaded: AppConfig = toml::from_str(&content)
            .map_err(|e| PlatformError::Config(format!("Failed to parse config: {}", e)))?;
        info!("Loaded configuration from {:?}", config_path);
        loaded
    };
    
    // Merge with defaults for any missing fields
    config.merge_defaults();
    
    Ok(config)
}

/// Save configuration to disk (only if changed)
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    let content = toml::to_string_pretty(config)
        .map_err(|e| PlatformError::Config(format!("Failed to serialize config: {}", e)))?;

    // Check if config file exists and has same content
    if config_path.exists() {
        if let Ok(existing) = std::fs::read_to_string(&config_path) {
            if existing == content {
                info!("Config unchanged, skipping write");
                return Ok(());
            }
        }
    }

    std::fs::write(&config_path, &content)?;
    info!("Saved configuration to {:?}", config_path);
    Ok(())
}




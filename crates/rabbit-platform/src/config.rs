//! Platform-specific configuration storage

use super::{PlatformError, Result};
use rabbit_models::AppConfig;
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

/// Get the home directory
#[allow(dead_code)]
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
}

/// Load configuration from disk
pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    
    if !config_path.exists() {
        info!("Config file not found, using defaults");
        return Ok(AppConfig::default());
    }
    
    let content = std::fs::read_to_string(&config_path)?;
    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| PlatformError::Config(format!("Failed to parse config: {}", e)))?;
    
    info!("Loaded configuration from {:?}", config_path);
    Ok(config)
}

/// Save configuration to disk
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    let content = toml::to_string_pretty(config)
        .map_err(|e| PlatformError::Config(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(&config_path, content)?;
    info!("Saved configuration to {:?}", config_path);
    Ok(())
}



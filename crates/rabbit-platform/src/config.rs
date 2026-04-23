use super::{PlatformError, Result};
use rabbit_models::config::{AppConfig, ConfigValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE: &str = "rabbit.toml";

pub fn get_config_dir() -> Result<PathBuf> {
    let dir = if cfg!(windows) {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")))
    } else {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
    };
    
    let rabbit_dir = dir.join("rabbit");
    if !rabbit_dir.exists() {
        fs::create_dir_all(&rabbit_dir)?;
    }
    
    Ok(rabbit_dir)
}

pub fn get_data_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let rabbit_dir = dir.join("rabbit");
    if !rabbit_dir.exists() {
        fs::create_dir_all(&rabbit_dir)?;
    }
    Ok(rabbit_dir)
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }
    
    let content = fs::read_to_string(&config_path)?;
    
    let mut config: AppConfig = toml::from_str(&content)
        .map_err(|e| PlatformError::Config(format!("Failed to parse config: {}", e)))?;
    
    config.merge_defaults();
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    
    let content = toml::to_string_pretty(config)
        .map_err(|e| PlatformError::Config(format!("Failed to serialize config: {}", e)))?;
    
    fs::write(&config_path, content)?;
    
    Ok(())
}

pub fn update_config<F>(modifier: F) -> Result<()>
where
    F: FnOnce(&mut AppConfig),
{
    let mut config = load_config()?;
    modifier(&mut config);
    save_config(&config)
}
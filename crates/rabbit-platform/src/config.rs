use super::{PlatformError, Result};
use rabbit_models::config::*;
use std::path::PathBuf;

const CONFIG_FILE: &str = "config.toml";

pub fn get_config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| PlatformError::Config("Cannot find config directory".into()))?
        .join("rabbit");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn get_data_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| PlatformError::Config("Cannot find data directory".into()))?
        .join("rabbit");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }
    
    let content = std::fs::read_to_string(&config_path)?;
    let mut config: AppConfig = toml::from_str(&content)
        .map_err(|e| PlatformError::Config(format!("Failed to parse config: {}", e)))?;
    
    config.merge_defaults();
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    let content = toml::to_string_pretty(config)
        .map_err(|e| PlatformError::Config(format!("Failed to serialize config: {}", e)))?;

    if config_path.exists() {
        let original = std::fs::read_to_string(&config_path)?;
        if content == original {
            return Ok(());
        }
    }

    std::fs::write(&config_path, &content)?;
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

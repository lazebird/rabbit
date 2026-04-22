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
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("window_x") && line.contains('=') {
            if let Some(val) = line.split('=').nth(1).and_then(|s| s.trim().parse::<i32>().ok()) {
                config.window_x = Some(val);
            }
        }
        if line.starts_with("window_y") && line.contains('=') {
            if let Some(val) = line.split('=').nth(1).and_then(|s| s.trim().parse::<i32>().ok()) {
                config.window_y = Some(val);
            }
        }
        if line.starts_with("window_width") && line.contains('=') {
            if let Some(val) = line.split('=').nth(1).and_then(|s| s.trim().parse::<i32>().ok()) {
                config.window_width = Some(val);
            }
        }
        if line.starts_with("window_height") && line.contains('=') {
            if let Some(val) = line.split('=').nth(1).and_then(|s| s.trim().parse::<i32>().ok()) {
                config.window_height = Some(val);
            }
        }
    }
    
    config.merge_defaults();
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);
    
    if config_path.exists() {
        let original = std::fs::read_to_string(&config_path)?;
        let mut new_content = original.clone();
        let mut modified = false;
        
        if let Some(x) = config.window_x {
            if new_content.contains("window_x =") {
                new_content = new_content.lines()
                    .map(|line| {
                        if line.starts_with("window_x =") {
                            modified = true;
                            format!("window_x = {}", x)
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            } else {
                new_content = format!("{}window_x = {}\n", new_content.trim_end().trim_end_matches('\n'), x);
                modified = true;
            }
        }
        
        if let Some(y) = config.window_y {
            if new_content.contains("window_y =") {
                new_content = new_content.lines()
                    .map(|line| {
                        if line.starts_with("window_y =") {
                            modified = true;
                            format!("window_y = {}", y)
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            } else {
                new_content = format!("{}window_y = {}\n", new_content.trim_end().trim_end_matches('\n'), y);
                modified = true;
            }
        }
        
        if let Some(w) = config.window_width {
            if new_content.contains("window_width =") {
                new_content = new_content.lines()
                    .map(|line| {
                        if line.starts_with("window_width =") {
                            modified = true;
                            format!("window_width = {}", w)
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            } else {
                new_content = format!("{}window_width = {}\n", new_content.trim_end().trim_end_matches('\n'), w);
                modified = true;
            }
        }
        
        if let Some(h) = config.window_height {
            if new_content.contains("window_height =") {
                new_content = new_content.lines()
                    .map(|line| {
                        if line.starts_with("window_height =") {
                            modified = true;
                            format!("window_height = {}", h)
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            } else {
                new_content = format!("{}window_height = {}\n", new_content.trim_end().trim_end_matches('\n'), h);
                modified = true;
            }
        }
        
        if modified {
            std::fs::write(&config_path, new_content)?;
        }
    }
    
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

use super::{PlatformError, Result};
use rabbit_models::config::AppConfig;
use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

const CONFIG_FILE: &str = "rabbit.toml";

/// 全局配置缓存
static CONFIG_CACHE: OnceLock<RwLock<AppConfig>> = OnceLock::new();
/// 上次保存的内容，用于脏检查
static LAST_SAVED_CONTENT: OnceLock<RwLock<String>> = OnceLock::new();

fn get_config_cache() -> &'static RwLock<AppConfig> {
    CONFIG_CACHE.get_or_init(|| {
        let config = load_config_internal().unwrap_or_default();
        RwLock::new(config)
    })
}

fn get_last_saved_cache() -> &'static RwLock<String> {
    LAST_SAVED_CONTENT.get_or_init(|| RwLock::new(String::new()))
}

pub fn get_config_dir() -> Result<PathBuf> {
    let dir = if cfg!(windows) {
        std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")))
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

/// 内部加载函数，不走缓存
fn load_config_internal() -> Result<AppConfig> {
    let config_path = get_config_dir()?.join(CONFIG_FILE);

    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&config_path)?;

    // 初始化上次保存的内容
    if let Ok(mut last) = get_last_saved_cache().write() {
        *last = content.clone();
    }

    let mut config: AppConfig = toml::from_str(&content).map_err(|e| PlatformError::Config(format!("Failed to parse config: {}", e)))?;

    config.merge_defaults();
    Ok(config)
}

/// 对外接口：从缓存加载配置
pub fn load_config() -> Result<AppConfig> {
    let cache = get_config_cache().read().map_err(|_| PlatformError::Config("Lock poisoned".into()))?;
    Ok(cache.clone())
}

/// 对外接口：保存配置并执行脏检查
pub fn save_config(config: &AppConfig) -> Result<()> {
    let content = toml::to_string_pretty(config).map_err(|e| PlatformError::Config(format!("Failed to serialize config: {}", e)))?;

    // 脏检查：对比上次保存的内容
    {
        let last_saved = get_last_saved_cache().read().map_err(|_| PlatformError::Config("Lock poisoned".into()))?;
        if *last_saved == content {
            return Ok(());
        }
    }

    let config_path = get_config_dir()?.join(CONFIG_FILE);
    fs::write(&config_path, &content)?;

    // 更新缓存和脏检查内容
    if let Ok(mut last) = get_last_saved_cache().write() {
        *last = content;
    }
    if let Ok(mut cache) = get_config_cache().write() {
        *cache = config.clone();
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

// --- 统一获取接口 ---

pub fn get_string(module: &str, key: &str) -> Option<String> {
    get_config_cache().read().ok()?.modules.get_string(module, key)
}

pub fn get_integer(module: &str, key: &str) -> Option<i64> {
    get_config_cache().read().ok()?.modules.get_integer(module, key)
}

pub fn get_bool(module: &str, key: &str) -> Option<bool> {
    get_config_cache().read().ok()?.modules.get_bool(module, key)
}

pub fn get_array(module: &str, key: &str) -> Option<Vec<String>> {
    get_config_cache().read().ok()?.modules.get_array(module, key)
}

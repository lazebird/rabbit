use rabbit_models::config::{AppConfig, ConfigValue};
use rabbit_platform::config::save_config;
use rabbit_platform::Result;

pub struct AppViewModel {
    config: AppConfig,
}

impl AppViewModel {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn get_config(&self) -> AppConfig {
        self.config.clone()
    }

    pub fn set_config(&mut self, config: AppConfig) {
        self.config = config;
    }

    pub fn save(&self) -> Result<()> {
        save_config(&self.config)
    }

    pub fn get_string(&self, section: &str, key: &str) -> Option<String> {
        self.config.modules.get_string(section, key)
    }

    pub fn get_integer(&self, section: &str, key: &str) -> Option<i64> {
        self.config.modules.get_integer(section, key)
    }

    pub fn get_bool(&self, section: &str, key: &str) -> Option<bool> {
        self.config.modules.get_bool(section, key)
    }

    pub fn get_array(&self, section: &str, key: &str) -> Option<Vec<String>> {
        self.config.modules.get_array(section, key)
    }

    pub fn set_string(&mut self, section: &str, key: &str, value: String) {
        self.config.modules.insert(section, key, ConfigValue::String(value));
    }

    pub fn set_integer(&mut self, section: &str, key: &str, value: i64) {
        self.config.modules.insert(section, key, ConfigValue::Integer(value));
    }

    pub fn set_bool(&mut self, section: &str, key: &str, value: bool) {
        self.config.modules.insert(section, key, ConfigValue::Boolean(value));
    }

    pub fn set_array(&mut self, section: &str, key: &str, value: Vec<String>) {
        let arr: Vec<ConfigValue> = value.into_iter().map(ConfigValue::String).collect();
        self.config.modules.insert(section, key, ConfigValue::Array(arr));
    }

    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }

    pub fn update_and_save(&mut self, config: AppConfig) -> Result<()> {
        self.config = config;
        save_config(&self.config)
    }
}

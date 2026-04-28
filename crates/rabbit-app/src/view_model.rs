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

    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }

    pub fn save_settings(&mut self) -> Result<()> {
        save_config(&self.config)
    }

    pub fn update_and_save(&mut self, config: AppConfig) -> Result<()> {
        self.config = config;
        save_config(&self.config)
    }

    pub fn update_global(&mut self, language: String, theme: String, systray: bool, top: bool, autostart: bool, autoupdate: bool) -> Result<()> {
        let modules = &mut self.config.modules;
        modules.insert("global", "language", ConfigValue::String(language));
        modules.insert("global", "theme", ConfigValue::String(theme));
        modules.insert("global", "systray", ConfigValue::Boolean(systray));
        modules.insert("global", "top", ConfigValue::Boolean(top));
        modules.insert("global", "autostart", ConfigValue::Boolean(autostart));
        modules.insert("global", "autoupdate", ConfigValue::Boolean(autoupdate));
        save_config(&self.config)
    }

    pub fn update_last_tab(&mut self, tab: usize) -> Result<()> {
        self.config.modules.insert("global", "last_active_tab", ConfigValue::Integer(tab as i64));
        save_config(&self.config)
    }
}
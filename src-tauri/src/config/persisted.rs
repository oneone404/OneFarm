use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

fn default_button_timeout_secs() -> u64 {
    5
}

fn default_click_delay_ms() -> u64 {
    1000
}

fn default_match_threshold() -> u32 {
    25
}

fn default_harvest_interval_mins() -> u64 {
    30
}

fn default_harvest_loop_count() -> u32 {
    2
}

fn default_sell_loop_count() -> u32 {
    2
}

fn default_game_launch_delay_secs() -> u64 {
    60
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub selected_seeds: Vec<String>,
    #[serde(default = "default_button_timeout_secs")]
    pub button_timeout_secs: u64,
    #[serde(default = "default_click_delay_ms")]
    pub click_delay_ms: u64,
    #[serde(default = "default_match_threshold")]
    pub match_threshold: u32,
    
    #[serde(default = "default_harvest_interval_mins")]
    pub harvest_interval_mins: u64,
    #[serde(default = "default_harvest_loop_count")]
    pub harvest_loop_count: u32,
    #[serde(default = "default_sell_loop_count")]
    pub sell_loop_count: u32,

    #[serde(default = "default_true")]
    pub enable_buy_seeds: bool,
    #[serde(default = "default_true")]
    pub enable_harvest_sell: bool,
    #[serde(default = "default_true")]
    pub enable_auto_login: bool,
    #[serde(default = "default_game_launch_delay_secs")]
    pub game_launch_delay_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            selected_seeds: Vec::new(),
            button_timeout_secs: 5,
            click_delay_ms: 1000,
            match_threshold: 25,
            harvest_interval_mins: 30,
            harvest_loop_count: 2,
            sell_loop_count: 2,
            enable_buy_seeds: true,
            enable_harvest_sell: true,
            enable_auto_login: true,
            game_launch_delay_secs: 60,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let path = Path::new("config.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(&self) -> std::result::Result<(), String> {
        let path = Path::new("config.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}

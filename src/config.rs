use std::fs;

use once_cell::sync::OnceCell;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    pub image: ImageConfig,
}

/// Core settings
#[derive(Deserialize)]
pub struct BotConfig {
    /// Bot token
    pub token: Option<String>,
    /// Bot app id
    pub app_id: Option<u64>,
    #[serde(default = "default_cmd_prefix")]
    pub cmd_prefix: String,
}

fn default_cmd_prefix() -> String { String::from(".") }

/// Image encoding settings
#[derive(Deserialize)]
pub struct ImageConfig {
    /// Quality of webp encoding
    #[serde(default = "default_webp_quality")]
    pub webp_quality: f32,
}

fn default_webp_quality() -> f32 { 80.0 }

pub static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn read_config() {
    // read and deserialize the config file
    info!("Reading config from file ./config.toml");
    let configfile = fs::read("./config.toml").unwrap();
    let config: Config = toml::from_slice(&configfile).unwrap();

    if let Err(_) = CONFIG.set(config) {
        panic!("Config reading failed");
    };
}

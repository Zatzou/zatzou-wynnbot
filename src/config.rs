use std::fs;

use once_cell::sync::OnceCell;
use serde::Deserialize;
use tracing::{error, info};

/// Root structure for the bot configuration
#[derive(Deserialize)]
pub struct Config {
    /// Core configuration options
    pub bot: BotConfig,
    #[serde(default)]
    /// Config options that deal with images
    pub image: ImageConfig,
}

/// Core settings
#[derive(Deserialize)]
pub struct BotConfig {
    /// Bot token
    token: Option<String>,
    /// Prefix used for commands
    #[serde(default = "default_cmd_prefix")]
    pub cmd_prefix: String,
}

fn default_cmd_prefix() -> String {
    String::from(".")
}

impl BotConfig {
    /// Function for getting the token since it isn't optional and has to be checked
    pub fn get_token(&self) -> &str {
        if let Some(t) = &self.token {
            t
        } else {
            error!("No token set in config.toml");
            panic!();
        }
    }
}

/// Image encoding settings
#[derive(Deserialize)]
pub struct ImageConfig {
    /// Quality of webp encoding
    #[serde(default = "default_webp_quality")]
    pub webp_quality: f32,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self { webp_quality: 80.0 }
    }
}

fn default_webp_quality() -> f32 {
    80.0
}

/// Static for holding the config
static CONFIG: OnceCell<Config> = OnceCell::new();

/// Function for initially loading and parsing the config file
///
/// This function should only be called once
pub fn read_config() {
    // read and deserialize the config file
    info!("Reading config from file ./config.toml");

    // read the config file
    match fs::read("./config.toml") {
        Ok(data) => {
            // parse the config file
            match toml::from_slice::<Config>(&data) {
                Ok(config) => {
                    if let Err(_) = CONFIG.set(config) {
                        // this is unreachable since `read_config()` should only be called once at startup
                        error!("Attempted to write config but config is already written");
                    };
                }
                Err(why) => {
                    error!("failed parsing the config file:");
                    error!("{}", why);
                    panic!();
                }
            }
        }
        Err(why) => {
            error!("failed reading the config file, does it exist?");
            error!("{}", why);
            panic!();
        }
    }
}

/// Function for getting the config variables easily
pub fn get_config() -> &'static Config {
    if let Some(c) = CONFIG.get() {
        c
    } else {
        // this is unreachable unless something tries to read the config before it is initialized
        unreachable!("Reading config failed but it should not be able to fail");
    }
}

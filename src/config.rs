use std::fs;

use serde::Deserialize;
use tracing::{error, info};

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

/// Root structure for the bot configuration
#[derive(Deserialize)]
pub struct Config {
    /// Core configuration options
    pub bot: BotConfig,
    #[serde(default)]
    /// Options for commands
    pub commands: CmdConfig,
    #[serde(default)]
    /// Config options that deal with images
    pub image: ImageConfig,
}

/// Core settings
#[derive(Deserialize)]
pub struct BotConfig {
    /// Bot token
    token: Option<String>,
    /// Name of the bot
    pub name: String,
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

#[derive(Deserialize)]
pub struct CmdConfig {
    /// whenever to register the slash commands on startup
    /// Once slash commands are registered it may take some time for discord to unregister them
    #[serde(default = "default_true")]
    pub register_slash_cmds: bool,
    /// whenever to enable prefix commands
    #[serde(default = "default_false")]
    pub enable_prefix_cmds: bool,
    /// prefix for commands, only used when prefix commands are enabled
    #[serde(default = "default_cmd_prefix")]
    pub cmd_prefix: String,
}

impl Default for CmdConfig {
    fn default() -> Self {
        Self {
            register_slash_cmds: true,
            enable_prefix_cmds: false,
            cmd_prefix: ".".to_string(),
        }
    }
}

fn default_cmd_prefix() -> String {
    String::from(".")
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

/// Function for initially loading and parsing the config file
///
/// This function should only be called once
pub fn read_config() -> Config {
    // read and deserialize the config file
    info!("Reading config from file ./config.toml");

    // read the config file
    match fs::read("./config.toml") {
        Ok(data) => {
            // parse the config file
            match toml::from_slice::<Config>(&data) {
                Ok(config) => config,
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

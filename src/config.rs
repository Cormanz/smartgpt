use anyhow::Result;
use serde::{Serialize, Deserialize};

pub const CONFIG_LOCATION: &str = "smartgpt.toml";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AutoType {
    Assistant {
        token_limit: Option<u16>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub personality: String,
    pub auto_type: AutoType,
}

pub fn write_defaults() -> Result<Config> {
    let config = Config {
        personality: "A superintelligent AI.".to_string(),
        auto_type: AutoType::Assistant {
            token_limit: Some(400),
        }
    };

    std::fs::write(CONFIG_LOCATION, toml::to_string(&config)?)?;

    Ok(config)
}

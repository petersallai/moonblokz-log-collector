use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "api-key")]
    pub api_key: String,

    #[serde(rename = "hub-url")]
    pub hub_url: String,

    #[serde(rename = "log-file")]
    pub log_file: PathBuf,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        // Validate required fields
        if config.api_key.is_empty() {
            anyhow::bail!("api-key cannot be empty");
        }

        if config.hub_url.is_empty() {
            anyhow::bail!("hub-url cannot be empty");
        }

        Ok(config)
    }
}

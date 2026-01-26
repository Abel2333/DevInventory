use std::path::PathBuf;

use anyhow::{self, Context, Result};
use serde::{Deserialize, Serialize};

use crate::keymgr::MasterKeySource;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub key: KeyConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct KeyConfig {
    pub env_name: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Level: trace, debug, info, warn, error
    pub level: Option<String>,
}

/// The runtime config (final config)
pub struct Config {
    pub db_path: PathBuf,
    pub master_key_source: MasterKeySource,
}

impl Config {
    /// Priority: CLI arg > env > config file > default value
    pub fn build(
        cli_db_path: Option<PathBuf>,
        mut master_key_source: MasterKeySource,
        cli_env_name: Option<String>,
    ) -> Result<Self> {
        let config_file = Self::load_config_file()?;

        let db_path = cli_db_path // CLI arguments
            .or_else(|| {
                std::env::var("DEVINVENTORY_DB_PATH") // environment variable
                    .ok()
                    .map(PathBuf::from)
            })
            .or_else(
                || config_file.database.path.as_ref().map(PathBuf::from), // config file
            )
            .unwrap_or_else(|| Self::default_db_path().unwrap());

        let env_name = cli_env_name
            .or_else(|| config_file.key.env_name.clone())
            .unwrap_or_else(|| "DEVINVENTORY_DMK".to_string());

        master_key_source.env_name = Some(env_name);

        Ok(Self {
            db_path,
            master_key_source,
        })
    }

    fn load_config_file() -> Result<ConfigFile> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            return Ok(ConfigFile::default());
        }

        let content =
            std::fs::read_to_string(&config_path).context("Failed to read config file")?;

        toml::from_str(&content).context("Failed to parse config file")
    }

    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Cannot determine user config directory")?;

        Ok(config_dir.join("devinventory").join("config.toml"))
    }

    fn default_db_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Cannot determine user config directory")?;

        Ok(config_dir.join("devinventory").join("secrets.db"))
    }
}

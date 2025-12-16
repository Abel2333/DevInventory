use std::path::PathBuf;

use anyhow::{self, Context, Result};
use serde::{Deserialize, Serialize};

use crate::keymgr::MasterKeySource;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub keyring: KeyringConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct KeyringConfig {
    pub service: Option<String>,
    pub account: Option<String>,
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
    pub keyring_service: String,
    pub keyring_account: String,
}

impl Config {
    /// Priority: CLI arg > env > config file > default value
    pub fn build(cli_db_path: Option<PathBuf>, master_key_source: MasterKeySource) -> Result<Self> {
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

        let keyring_service = std::env::var("DEVINVENTORY_KEYRING_SERVICE")
            .ok()
            .or_else(|| config_file.keyring.service.clone())
            .unwrap_or_else(|| "devinventory".to_string());

        let keyring_account = std::env::var("DEVINVENTORY_KEYRING_ACCOUNT")
            .ok()
            .or_else(|| config_file.keyring.account.clone())
            .unwrap_or_else(|| "dmk".to_string());

        Ok(Self {
            db_path,
            master_key_source,
            keyring_service,
            keyring_account,
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

    pub fn generate_example_config() -> String {
        let example = ConfigFile {
            database: DatabaseConfig {
                path: Some("/custom/path/to/secrets.db".to_string()),
            },
            keyring: KeyringConfig {
                service: Some("devinventory".to_string()),
                account: Some("dmk".to_string()),
            },
            logging: LoggingConfig {
                level: Some("info".to_string()),
            },
        };

        toml::to_string_pretty(&example).unwrap()
    }
}

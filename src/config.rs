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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct EnvGuard {
        key: &'static str,
        prev: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = env::var(key).ok();
            unsafe {
                env::set_var(key, value);
            }
            Self { key, prev }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(prev) = self.prev.as_ref() {
                unsafe {
                    env::set_var(self.key, prev);
                }
            } else {
                unsafe {
                    env::remove_var(self.key);
                }
            }
        }
    }

    fn write_config(temp_dir: &TempDir, content: &str) -> Result<()> {
        let cfg_dir = temp_dir.path().join("devinventory");
        fs::create_dir_all(&cfg_dir)?;
        fs::write(cfg_dir.join("config.toml"), content)?;
        Ok(())
    }

    #[test]
    fn build_prefers_cli_over_env_and_config() {
        let _lock = env_lock();
        let temp_dir = TempDir::new().unwrap();
        let _cfg_home = EnvGuard::set("XDG_CONFIG_HOME", temp_dir.path().to_str().unwrap());
        let _home = EnvGuard::set("HOME", temp_dir.path().to_str().unwrap());
        let _appdata = EnvGuard::set("APPDATA", temp_dir.path().to_str().unwrap());
        let env_db_path = temp_dir.path().join("env.db");
        let _env_db = EnvGuard::set("DEVINVENTORY_DB_PATH", env_db_path.to_str().unwrap());

        let cli_db_path = temp_dir.path().join("cli.db");

        write_config(
            &temp_dir,
            r#"
                [database]
                path = "config.db"
                [key]
                env_name = "FROM_CONFIG"
            "#,
        )
        .unwrap();

        let master_key_source = MasterKeySource {
            base64_inline: None,
            env_name: None,
        };
        let config = Config::build(
            Some(cli_db_path.clone()),
            master_key_source,
            Some("CLI_ENV".to_string()),
        )
        .unwrap();

        assert_eq!(config.db_path, cli_db_path);
        assert_eq!(
            config.master_key_source.env_name.as_deref(),
            Some("CLI_ENV")
        );
    }

    #[test]
    fn build_uses_env_then_config_then_default() {
        let _lock = env_lock();
        let temp_dir = TempDir::new().unwrap();
        let _cfg_home = EnvGuard::set("XDG_CONFIG_HOME", temp_dir.path().to_str().unwrap());
        let _home = EnvGuard::set("HOME", temp_dir.path().to_str().unwrap());
        let _appdata = EnvGuard::set("APPDATA", temp_dir.path().to_str().unwrap());
        let env_db_path = temp_dir.path().join("env.db");
        let _env_db = EnvGuard::set("DEVINVENTORY_DB_PATH", env_db_path.to_str().unwrap());

        write_config(
            &temp_dir,
            r#"
                [database]
                path = "config.db"
                [key]
                env_name = "FROM_CONFIG"
            "#,
        )
        .unwrap();

        let master_key_source = MasterKeySource {
            base64_inline: None,
            env_name: None,
        };
        let config = Config::build(None, master_key_source, None).unwrap();

        assert_eq!(config.db_path, env_db_path);
        assert_eq!(
            config.master_key_source.env_name.as_deref(),
            Some("FROM_CONFIG")
        );
    }
}

use std::env;

use crate::{crypto::MasterKey, error::KeyError};
use base64::{Engine as _, engine::general_purpose};
use log::{error, info};
use rand::RngCore;
use zeroize::Zeroize;

pub enum KeyResult {
    Existing(MasterKey),
    Generated(MasterKey),
}

impl KeyResult {
    pub fn key(&self) -> &MasterKey {
        match self {
            Self::Existing(k) | Self::Generated(k) => k,
        }
    }

    pub fn into_key(self) -> MasterKey {
        match self {
            Self::Existing(k) | Self::Generated(k) => k,
        }
    }

    pub fn key_b64(&self) -> String {
        let key = self.key();

        general_purpose::STANDARD.encode(key.as_bytes())
    }
}

/// DMK only allowed from cli inline or env variable
#[derive(Clone)]
pub struct MasterKeySource {
    pub base64_inline: Option<String>,
    pub env_name: Option<String>,
}

pub struct MasterKeyProvider {
    src: MasterKeySource,
}

impl MasterKeyProvider {
    pub fn new(src: MasterKeySource) -> Self {
        Self { src }
    }

    /// Obtain existing master key. If `generate_if_missing` is true, will create a new key.
    pub fn obtain(&self, generate_if_missing: bool) -> Result<KeyResult, KeyError> {
        // Step 1. Try to get existing key
        if let Some(b64) = self.src.base64_inline.as_ref() {
            let key = decode_key(b64)?;
            info!("master key provided via CLI");
            return Ok(KeyResult::Existing(key));
        }

        if let Some(env_name) = self.src.env_name.as_ref() {
            match env::var(env_name) {
                Ok(val) => {
                    let key = decode_key(&val)?;
                    info!("master key loaded from environment variable '{}'", env_name);
                    return Ok(KeyResult::Existing(key));
                }
                Err(env::VarError::NotPresent) => {
                    error!("environment variable '{}' not set", env_name);
                }
                Err(e) => {
                    return Err(KeyError::EnvVar(e));
                }
            }
        }

        // Step 2. Try to generate key if allowed
        if !generate_if_missing {
            return Err(KeyError::Missing);
        }

        info!("generated new master key");
        Ok(KeyResult::Generated(generate_key()))
    }

    pub fn rotate(&self) -> Result<KeyResult, KeyError> {
        info!("rotating master key");
        Ok(KeyResult::Generated(generate_key()))
    }
}

fn decode_key(b64: &str) -> Result<MasterKey, KeyError> {
    let mut bytes = general_purpose::STANDARD
        .decode(b64.trim())
        .map_err(KeyError::InvalidBase64)?;
    if bytes.len() != 32 {
        return Err(KeyError::InvalidLength(bytes.len()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    bytes.zeroize();
    Ok(MasterKey(arr))
}

fn generate_key() -> MasterKey {
    let mut key = [0u8; 32];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut key);
    MasterKey(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose;
    use std::env;
    use std::sync::{Mutex, OnceLock};

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

        fn unset(key: &'static str) -> Self {
            let prev = env::var(key).ok();
            unsafe {
                env::remove_var(key);
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

    #[test]
    fn obtain_prefers_inline_key_over_env() {
        let _lock = env_lock();
        let env_key = general_purpose::STANDARD.encode([2u8; 32]);
        let _env = EnvGuard::set("TEST_DMK", &env_key);

        let inline_key = general_purpose::STANDARD.encode([1u8; 32]);
        let provider = MasterKeyProvider::new(MasterKeySource {
            base64_inline: Some(inline_key),
            env_name: Some("TEST_DMK".to_string()),
        });

        let result = provider.obtain(false).unwrap();
        assert_eq!(result.key().as_bytes(), &[1u8; 32]);
    }

    #[test]
    fn obtain_uses_env_when_inline_missing() {
        let _lock = env_lock();
        let env_key = general_purpose::STANDARD.encode([3u8; 32]);
        let _env = EnvGuard::set("TEST_DMK", &env_key);

        let provider = MasterKeyProvider::new(MasterKeySource {
            base64_inline: None,
            env_name: Some("TEST_DMK".to_string()),
        });

        let result = provider.obtain(false).unwrap();
        assert_eq!(result.key().as_bytes(), &[3u8; 32]);
    }

    #[test]
    fn obtain_errors_when_missing_and_no_generate() {
        let _lock = env_lock();
        let _env = EnvGuard::unset("TEST_DMK");

        let provider = MasterKeyProvider::new(MasterKeySource {
            base64_inline: None,
            env_name: Some("TEST_DMK".to_string()),
        });

        let err = provider.obtain(false).err().unwrap();
        assert!(matches!(err, KeyError::Missing));
    }

    #[test]
    fn obtain_errors_on_invalid_env_value() {
        let _lock = env_lock();
        let _env = EnvGuard::set("TEST_DMK", "not-base64");

        let provider = MasterKeyProvider::new(MasterKeySource {
            base64_inline: None,
            env_name: Some("TEST_DMK".to_string()),
        });

        let err = provider.obtain(false).err().unwrap();
        assert!(matches!(err, KeyError::InvalidBase64(_)));
    }

    #[test]
    fn rotate_generates_new_key() {
        let provider = MasterKeyProvider::new(MasterKeySource {
            base64_inline: None,
            env_name: None,
        });

        let result = provider.rotate().unwrap();
        assert_eq!(result.key().as_bytes().len(), 32);
        assert_ne!(result.key().as_bytes(), &[0u8; 32]);
    }
}

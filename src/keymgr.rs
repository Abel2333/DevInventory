use std::env;

use crate::crypto::MasterKey;
use anyhow::{Context, Result, anyhow};
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
    pub fn obtain(&self, generate_if_missing: bool) -> Result<KeyResult> {
        // Step 1. Try to get existing key
        if let Some(b64) = self.src.base64_inline.as_ref() {
            let key = decode_key(b64).context("invalid master key from --dmk")?;
            info!("master key provided via CLI");
            return Ok(KeyResult::Existing(key));
        }

        if let Some(env_name) = self.src.env_name.as_ref() {
            match env::var(env_name) {
                Ok(val) => {
                    let key =
                        decode_key(&val).context("invalid master key in environment variable")?;
                    info!("master key loaded from environment variable '{}'", env_name);
                    return Ok(KeyResult::Existing(key));
                }
                Err(env::VarError::NotPresent) => {
                    error!("environment variable '{}' not set", env_name);
                }
                Err(e) => {
                    return Err(anyhow!(e).context("failed to read environment variable"));
                }
            }
        }

        // Step 2. Try to generate key if allowed
        if !generate_if_missing {
            return Err(anyhow!(
                "master key not found; provide --dmk or set env var"
            ));
        }

        info!("generated new master key");
        Ok(KeyResult::Generated(generate_key()))
    }

    pub fn rotate(&self) -> Result<KeyResult> {
        info!("rotating master key");
        Ok(KeyResult::Generated(generate_key()))
    }
}

fn decode_key(b64: &str) -> Result<MasterKey> {
    let mut bytes = general_purpose::STANDARD
        .decode(b64.trim())
        .map_err(|_| anyhow!("invalid base64 master key"))?;
    if bytes.len() != 32 {
        return Err(anyhow!("master key must be 32 bytes"));
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

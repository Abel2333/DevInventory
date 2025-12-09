use crate::crypto::MasterKey;
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use keyring::Entry;
use log::{debug, info};
use rand::RngCore;
use zeroize::Zeroize;

const SERVICE: &str = "devinventory";
const ACCOUNT: &str = "dmk";

pub struct MasterKeySource {
    pub base64_inline: Option<String>,
    pub allow_keyring: bool,
}

pub struct MasterKeyProvider {
    src: MasterKeySource,
}

impl MasterKeyProvider {
    pub fn new(src: MasterKeySource) -> Self {
        Self { src }
    }

    pub async fn obtain(&self) -> Result<MasterKey> {
        if let Some(k) = self
            .src
            .base64_inline
            .as_ref()
            .and_then(|b| decode_key(b).ok())
        {
            info!("master key provided inline");
            return Ok(k);
        }

        if self.src.allow_keyring
            && let Some(k) = self.read_keyring()?
        {
            info!("master key loaded from keyring");
            return Ok(k);
        }

        let key = generate_key();
        let encoded = general_purpose::STANDARD.encode(&key.0);
        println!(
            "Generated new master key (base64). Save this now: {}",
            encoded
        );
        if self.src.allow_keyring {
            self.write_keyring(&encoded)?;
            info!("new master key written to keyring");
            println!("Stored in OS keyring under service '{SERVICE}' account '{ACCOUNT}'.");
        } else {
            println!("Not stored in keyring (--no-keyring). You must manage it manually.");
        }
        Ok(key)
    }

    pub async fn rotate(&self) -> Result<MasterKey> {
        let key = generate_key();
        let encoded = general_purpose::STANDARD.encode(&key.0);
        println!("New master key (base64). Save immediately: {}", encoded);
        if self.src.allow_keyring {
            self.write_keyring(&encoded)?;
            println!("Keyring updated.");
            info!("keyring updated during rotation");
        }
        Ok(key)
    }

    fn read_keyring(&self) -> Result<Option<MasterKey>> {
        let entry = Entry::new(SERVICE, ACCOUNT)?;
        match entry.get_password() {
            Ok(value) => decode_key(&value).map(Some),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => {
                debug!("keyring read error: {e:?}");
                Err(anyhow!(e)).context("reading keyring")
            }
        }
    }

    fn write_keyring(&self, encoded: &str) -> Result<()> {
        let entry = Entry::new(SERVICE, ACCOUNT)?;
        entry.set_password(encoded).context("writing keyring")?;
        Ok(())
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

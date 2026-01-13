use crate::{
    crypto::{MasterKey, SecretCrypto},
    keymgr::MasterKeyProvider,
};
use anyhow::Result;

pub struct CryptoService {
    master_key: MasterKey,
}

impl CryptoService {
    pub async fn new(key_provider: &MasterKeyProvider, generate_new: bool) -> Result<Self> {
        let master_key = key_provider.obtain(generate_new).await?;
        Ok(Self { master_key })
    }

    /// Encrypt data
    pub fn encrypt(&self, name: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let secret_crypto = SecretCrypto::new(self.master_key.clone());

        secret_crypto.encrypt(name, plaintext)
    }

    /// Decrypt data
    pub fn decrypt(&self, name: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let secret_crypto = SecretCrypto::new(self.master_key.clone());

        secret_crypto.decrypt(name, ciphertext)
    }

    pub fn create_secret_crypto(&self) -> SecretCrypto {
        SecretCrypto::new(self.master_key.clone())
    }

    pub fn master_key(&self) -> &MasterKey {
        &self.master_key
    }
}

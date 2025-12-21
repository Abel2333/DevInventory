use crate::{
    crypto::MasterKey,
    crypto_service::CryptoService,
    db::Repository,
    domain::{Secret, SecretMetadata},
    keymgr::MasterKeyProvider,
};
use anyhow::Result;
use std::path::Path;

pub struct SecretService {
    repo: Repository,
    crypto_service: CryptoService,
}

impl SecretService {
    /// Initialize a new devinventory project
    /// Creates database, runs migrations, and generates a new master key
    pub async fn init(
        db_path: &Path,
        key_provider: &MasterKeyProvider,
    ) -> Result<(Self, MasterKey)> {
        // Create database and run migrations
        let repo = Repository::connect(db_path).await?;
        repo.migrate().await?;

        // Generate new master key
        let crypto_service = CryptoService::new(key_provider, true).await?;
        let master_key = crypto_service.master_key().clone();

        Ok((Self { repo, crypto_service }, master_key))
    }

    pub fn new(repo: Repository, crypto_service: CryptoService) -> Self {
        Self {
            repo,
            crypto_service,
        }
    }

    pub async fn add_secret(
        &self,
        name: String,
        value: Vec<u8>,
        kind: Option<String>,
        note: Option<String>,
    ) -> Result<Secret> {
        let ciphertext = self.crypto_service.encrypt(&name, &value)?;

        let record = self
            .repo
            .upsert_secret(&name, kind, note, &ciphertext)
            .await?;

        Ok(Secret {
            id: record.id,
            name: record.name,
            kind: record.kind,
            note: record.note,
            plaintext: value,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    /// Acquire the secret key
    pub async fn get_secret(&self, name: &str) -> Result<Secret> {
        let record = if let Some(record) = self.repo.fetch_secret(name).await? {
            record
        } else {
            return Err(anyhow::anyhow!("secret not found"));
        };

        let plaintext = self
            .crypto_service
            .decrypt(&record.name, &record.ciphertext)?;

        Ok(Secret {
            id: record.id,
            name: record.name,
            kind: record.kind,
            note: record.note,
            plaintext,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    /// List all secrets in Vec type
    pub async fn list_secrets(&self) -> Result<Vec<SecretMetadata>> {
        let secrets = self.repo.list_secrets().await?;
        let metadata = secrets
            .into_iter()
            .map(|record| SecretMetadata {
                id: record.id,
                name: record.name,
                kind: record.kind,
                note: record.note,
                created_at: record.created_at,
                updated_at: record.updated_at,
            })
            .collect();

        Ok(metadata)
    }

    /// Search Secrets
    pub async fn search_secrets(&self, query: &str) -> Result<Vec<SecretMetadata>> {
        let secrets = self.repo.search_secrets(query).await?;

        let searched_secrets = secrets
            .into_iter()
            .map(|record| SecretMetadata {
                id: record.id,
                name: record.name,
                kind: record.kind,
                note: record.note,
                created_at: record.created_at,
                updated_at: record.updated_at,
            })
            .collect();

        Ok(searched_secrets)
    }

    /// Delete Secret
    pub async fn delete_secret(&self, name: &str) -> Result<()> {
        self.repo.delete_secret(name).await?;

        Ok(())
    }

    /// Change the Master Key
    pub async fn rotate_master_key(&self, new_crypto_service: CryptoService) -> Result<()> {
        // Create SecretCrypto instructions
        let old_crypto = self.crypto_service.create_secret_crypto();
        let new_crypto = new_crypto_service.create_secret_crypto();

        self.repo.reencrypt_all(&old_crypto, &new_crypto).await
    }
}

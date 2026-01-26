use crate::{
    crypto::MasterKey,
    crypto::CryptoService,
    domain::{Secret, SecretMetadata},
    keymgr::MasterKeyProvider,
    storage::Repository,
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
        // NOTE:The generated result should be show by UI
        let key_result = key_provider.obtain(true)?;
        let crypto_service = CryptoService::new(key_result.into_key());
        let master_key = crypto_service.master_key().clone();

        Ok((
            Self {
                repo,
                crypto_service,
            },
            master_key,
        ))
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
        let metadata = secrets.into_iter().map(SecretMetadata::from).collect();

        Ok(metadata)
    }

    /// Search Secrets
    pub async fn search_secrets(&self, query: &str) -> Result<Vec<SecretMetadata>> {
        let secrets = self.repo.search_secrets(query).await?;

        let searched_secrets = secrets.into_iter().map(SecretMetadata::from).collect();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::MasterKey;
    use crate::storage::Repository;
    use tempfile::TempDir;

    #[tokio::test]
    async fn add_get_list_search_delete() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("secrets.db");
        let repo = Repository::connect(&db_path).await.unwrap();
        repo.migrate().await.unwrap();

        let crypto = CryptoService::new(MasterKey([1u8; 32]));
        let service = SecretService::new(repo, crypto);

        service
            .add_secret(
                "api".to_string(),
                b"secret-token".to_vec(),
                Some("token".to_string()),
                Some("prod".to_string()),
            )
            .await
            .unwrap();

        let secret = service.get_secret("api").await.unwrap();
        assert_eq!(secret.plaintext, b"secret-token");

        let list = service.list_secrets().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "api");

        let search = service.search_secrets("prod").await.unwrap();
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].name, "api");

        service.delete_secret("api").await.unwrap();
        let list_after = service.list_secrets().await.unwrap();
        assert!(list_after.is_empty());
    }

    #[tokio::test]
    async fn rotate_master_key_allows_new_service_to_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("secrets.db");
        let repo = Repository::connect(&db_path).await.unwrap();
        repo.migrate().await.unwrap();

        let crypto_old = CryptoService::new(MasterKey([1u8; 32]));
        let service = SecretService::new(repo, crypto_old);

        service
            .add_secret("db".to_string(), b"conn-string".to_vec(), None, None)
            .await
            .unwrap();

        let crypto_new = CryptoService::new(MasterKey([2u8; 32]));
        service.rotate_master_key(crypto_new).await.unwrap();

        let repo2 = Repository::connect(&db_path).await.unwrap();
        repo2.migrate().await.unwrap();
        let service2 = SecretService::new(repo2, CryptoService::new(MasterKey([2u8; 32])));

        let secret = service2.get_secret("db").await.unwrap();
        assert_eq!(secret.plaintext, b"conn-string");
    }
}

use crate::crypto::SecretCrypto;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, sqlite::SqlitePoolOptions};
use std::{fs, fs::OpenOptions, path::Path};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretRecord {
    pub id: Uuid,
    pub name: String,
    pub kind: Option<String>,
    pub note: Option<String>,
    pub ciphertext: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Repository {
    pool: Pool<Sqlite>,
}

impl Repository {
    pub async fn connect(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            // Touch the file so SQLite doesn't fail with code 14 on some sandboxed FS.
            OpenOptions::new().create(true).write(true).open(path)?;
            info!("created new database file at {}", path.to_string_lossy());
        }
        let url = format!("sqlite://{}", path.to_string_lossy());
        debug!("connecting sqlite at {}", url);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .context("connect sqlite")?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS secrets (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL UNIQUE,
                kind        TEXT,
                note        TEXT,
                ciphertext  BLOB NOT NULL,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_secrets_kind ON secrets(kind);")
            .execute(&self.pool)
            .await?;
        debug!("database schema ensured");
        Ok(())
    }

    pub async fn upsert_secret(
        &self,
        name: &str,
        kind: Option<String>,
        note: Option<String>,
        ciphertext: &[u8],
    ) -> Result<SecretRecord> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO secrets (id, name, kind, note, ciphertext, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(name) DO UPDATE SET
                kind=excluded.kind,
                note=excluded.note,
                ciphertext=excluded.ciphertext,
                updated_at=excluded.updated_at
            RETURNING *
            "#,
        )
        .bind(id.to_string())
        .bind(name)
        .bind(&kind)
        .bind(&note)
        .bind(ciphertext)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        info!("upserted secret '{}'", name);

        Ok(SecretRecord {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap_or_else(|_| Uuid::nil()),
            name: row.get("name"),
            kind,
            note,
            ciphertext: row.get("ciphertext"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn fetch_secret(&self, name: &str) -> Result<Option<SecretRecord>> {
        let row = sqlx::query(
            r#"SELECT id, name, kind, note, ciphertext, created_at, updated_at FROM secrets WHERE name = ?1"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        debug!(
            "fetch secret '{}' -> {}",
            name,
            if row.is_some() { "hit" } else { "miss" }
        );
        Ok(row.map(|r| SecretRecord {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap_or_else(|_| Uuid::nil()),
            name: r.get("name"),
            kind: r.get("kind"),
            note: r.get("note"),
            ciphertext: r.get("ciphertext"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    pub async fn list_secrets(&self) -> Result<Vec<SecretRecord>> {
        let rows = sqlx::query(
            r#"SELECT id, name, kind, note, ciphertext, created_at, updated_at FROM secrets ORDER BY name"#
        )
        .fetch_all(&self.pool)
        .await?;
        debug!("list_secrets returned {} rows", rows.len());
        Ok(rows
            .into_iter()
            .map(|r| SecretRecord {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::nil()),
                name: r.get("name"),
                kind: r.get("kind"),
                note: r.get("note"),
                ciphertext: r.get("ciphertext"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Search name/kind/note with a case-insensitive substring match.
    pub async fn search_secrets(&self, query: &str) -> Result<Vec<SecretRecord>> {
        let pattern = format!("%{}%", query.to_lowercase());
        let rows = sqlx::query(
            r#"SELECT id, name, kind, note, ciphertext, created_at, updated_at
               FROM secrets
               WHERE lower(name) LIKE ?1 OR lower(kind) LIKE ?1 OR lower(note) LIKE ?1
               ORDER BY name"#,
        )
        .bind(pattern)
        .fetch_all(&self.pool)
        .await?;
        info!("search_secrets '{}' -> {} rows", query, rows.len());
        Ok(rows
            .into_iter()
            .map(|r| SecretRecord {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::nil()),
                name: r.get("name"),
                kind: r.get("kind"),
                note: r.get("note"),
                ciphertext: r.get("ciphertext"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    pub async fn delete_secret(&self, name: &str) -> Result<bool> {
        let res = sqlx::query("DELETE FROM secrets WHERE name = ?1")
            .bind(name)
            .execute(&self.pool)
            .await?;
        debug!("delete_secret '{}' -> {}", name, res.rows_affected());
        Ok(res.rows_affected() > 0)
    }

    pub async fn reencrypt_all(
        &self,
        old_crypto: &SecretCrypto,
        new_crypto: &SecretCrypto,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query(r#"SELECT id, name, ciphertext FROM secrets"#)
            .fetch_all(&mut *tx)
            .await?;
        let total = rows.len();

        for row in rows {
            let name: String = row.get("name");
            let ct: Vec<u8> = row.get("ciphertext");
            let id: String = row.get("id");

            let plaintext = old_crypto.decrypt(&name, &ct)?;
            let new_ct = new_crypto.encrypt(&name, &plaintext)?;
            sqlx::query("UPDATE secrets SET ciphertext = ?1, updated_at = ?2 WHERE id = ?3")
                .bind(new_ct)
                .bind(Utc::now())
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        info!("re-encrypted {} secrets with new master key", total);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::MasterKey;
    use crate::crypto::SecretCrypto;
    use crate::db::Repository;
    use std::path::PathBuf;

    #[tokio::test]
    async fn repo_crud_and_rotate() {
        // use in-memory sqlite to avoid filesystem writes in tests
        let db_path = PathBuf::from(":memory:");

        let repo = Repository::connect(&db_path).await.unwrap();
        repo.migrate().await.unwrap();

        let key1 = MasterKey([1u8; 32]);
        let crypto1 = SecretCrypto::new(key1.clone());

        // create
        let ct = crypto1.encrypt("api", b"secret-token").unwrap();
        repo.upsert_secret("api", Some("token".into()), None, &ct)
            .await
            .unwrap();

        // read
        let rec = repo.fetch_secret("api").await.unwrap().unwrap();
        let pt = crypto1.decrypt(&rec.name, &rec.ciphertext).unwrap();
        assert_eq!(pt, b"secret-token");

        // rotate
        let key2 = MasterKey([2u8; 32]);
        let crypto2 = SecretCrypto::new(key2.clone());
        repo.reencrypt_all(&crypto1, &crypto2).await.unwrap();
        let rec2 = repo.fetch_secret("api").await.unwrap().unwrap();
        let pt2 = crypto2.decrypt(&rec2.name, &rec2.ciphertext).unwrap();
        assert_eq!(pt2, b"secret-token");

        // delete
        assert!(repo.delete_secret("api").await.unwrap());
        assert!(repo.fetch_secret("api").await.unwrap().is_none());
    }
}

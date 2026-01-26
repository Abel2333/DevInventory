use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::db::SecretRecord;

// Data after decryption
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Secret {
    pub id: Uuid,
    pub name: String,
    pub kind: Option<String>,
    pub note: Option<String>,
    pub plaintext: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Metadata without secretion
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SecretMetadata {
    pub id: Uuid,
    pub name: String,
    pub kind: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Secret {
    /// Convert to metadata
    pub fn to_metadata(&self) -> SecretMetadata {
        SecretMetadata {
            id: self.id,
            name: self.name.clone(),
            kind: self.kind.clone(),
            note: self.note.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl From<SecretRecord> for SecretMetadata {
    fn from(record: SecretRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            kind: record.kind,
            note: record.note,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

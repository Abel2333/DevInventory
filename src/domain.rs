use chrono::{DateTime, Utc};
use uuid::Uuid;

// Data after decryption
#[derive(Debug, Clone)]
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

use tabled::Tabled;

use crate::domain::SecretMetadata;

pub fn mask(plaintext: &[u8]) -> String {
    if plaintext.is_empty() {
        return "(empty)".to_string();
    }

    let s = String::from_utf8_lossy(plaintext);
    let len = s.chars().count();
    let head = s.chars().take(2).collect::<String>();
    let tail = s.chars().rev().take(2).collect::<String>();

    match len {
        0 => "(empty)".into(),
        1..=3 => "***".into(),
        _ => format!("{}***{}", head, tail.chars().rev().collect::<String>()),
    }
}

#[derive(Tabled)]
pub struct SecretRow {
    pub name: String,
    pub kind: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<SecretMetadata> for SecretRow {
    fn from(value: SecretMetadata) -> Self {
        Self {
            name: value.name,
            kind: value.kind.unwrap_or_default(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

impl SecretRow {
    pub fn from_metadata_list(metadata_list: Vec<SecretMetadata>) -> Vec<Self> {
        metadata_list.into_iter().map(Self::from).collect()
    }
}

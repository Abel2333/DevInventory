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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn mask_handles_various_lengths() {
        assert_eq!(mask(b""), "(empty)");
        assert_eq!(mask(b"a"), "***");
        assert_eq!(mask(b"ab"), "***");
        assert_eq!(mask(b"abc"), "***");
        assert_eq!(mask(b"abcd"), "ab***cd");
        assert_eq!(mask(b"abcdef"), "ab***ef");
    }

    #[test]
    fn secret_row_converts_metadata() {
        let ts = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        let metadata = SecretMetadata {
            id: Uuid::new_v4(),
            name: "api".to_string(),
            kind: None,
            note: None,
            created_at: ts,
            updated_at: ts,
        };

        let rows = SecretRow::from_metadata_list(vec![metadata]);
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.name, "api");
        assert_eq!(row.kind, "");
        assert_eq!(row.created_at, ts.to_rfc3339());
        assert_eq!(row.updated_at, ts.to_rfc3339());
    }
}

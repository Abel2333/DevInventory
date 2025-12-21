use std::u8;

use tabled::Tabled;

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

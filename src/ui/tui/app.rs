use std::time::Instant;

use uuid::Uuid;

use crate::{
    domain::{Secret, SecretMetadata},
    ui::tui::commands::Command,
};

/// Indicate the current screen
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Mode {
    #[default]
    List,
    Detail,
    Search,
    AddForm,
    EditForm,
    ConfirmDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormField {
    #[default]
    Name,
    Kind,
    Note,
    Plaintext,
}

/// Form Data
#[derive(Debug, Clone, Default)]
pub struct SecretForm {
    pub id: Option<Uuid>, // `None` when add it, `Some` when edit it.
    pub name: String,
    pub kind: Option<String>,
    pub note: Option<String>,
    pub plaintext: String, // User-entered plaintext (not Vec<u8>, for easier editing)
    pub cursor_field: FormField, // 0=name, 1=kind, 2=note, 3=plaintext
    pub dirty: bool,
}

impl SecretForm {
    // Create form from `Secret`
    pub fn from_secret(secret: &Secret) -> Self {
        Self {
            id: Some(secret.id),
            name: secret.name.clone(),
            kind: secret.kind.clone(),
            note: secret.note.clone(),
            plaintext: String::from_utf8_lossy(&secret.plaintext).to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    // --- Core Parameters ---
    pub mode: Mode,
    pub secrets: Vec<SecretMetadata>,
    pub list_index: usize,      // currently Selected Item
    pub status: Option<String>, // status line message
    pub last_tick: Instant,

    // --- List Mode ---
    pub scroll_offset: usize,
    pub page_size: usize,

    // --- Detail Mode ---
    pub current_secret: Option<Secret>,

    // --- Search Mode ---
    pub search_query: String,
    pub search_cursor: usize,
    pub filtered_indices: Vec<usize>, // index to the secrets

    // --- Add/Edit Mode ---
    pub form: Option<SecretForm>,

    // --- ConfirmDelete Mode ---
    pub pending_delete_id: Option<Uuid>,
}

impl AppState {
    pub fn new(secrets: Vec<SecretMetadata>) -> Self {
        Self {
            mode: Mode::default(),
            secrets,
            list_index: 0,
            status: None,
            last_tick: Instant::now(),
            scroll_offset: 0,
            page_size: 10,
            current_secret: None,
            search_query: String::new(),
            search_cursor: 0,
            filtered_indices: Vec::new(),
            form: None,
            pending_delete_id: None,
        }
    }

    fn transition(state: Mode, cmd: Command) -> Mode {
        match (state, cmd) {
            (Mode::List, Command::Open) => Mode::Detail,
            (Mode::List, Command::SearchStart) => Mode::Search,
            (Mode::List, Command::Add) => Mode::AddForm,
            (Mode::List, Command::Edit) => Mode::EditForm,
            (Mode::List, Command::Delete) => Mode::ConfirmDelete,
            (Mode::List, Command::MoveUp) => Mode::List,
            (Mode::List, Command::MoveDown) => Mode::List,
            (Mode::List, Command::PageUp) => Mode::List,
            (Mode::List, Command::PageDown) => Mode::List,
            (Mode::List, Command::Quit) =>
            (s, _) => s,
        }
    }
}

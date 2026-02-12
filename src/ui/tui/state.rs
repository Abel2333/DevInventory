use std::time::Instant;

use uuid::Uuid;

use crate::{
    domain::{Secret, SecretMetadata},
    ui::tui::commands::Command,
};

/// Indicate the current screen
#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub enum Mode {
    #[default]
    List,
    Detail,
    Search,
    AddForm,
    EditForm,
    ConfirmDelete,
    Exit,
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

    pub fn prepare_edit_form(&mut self, secret: &Secret) {
        self.form = Some(SecretForm::from_secret(secret));
    }

    pub fn clear_form(&mut self) {
        self.form = None;
    }

    fn transition(state: Mode, cmd: Command) -> Mode {
        match (state, cmd) {
            // From `List`
            (Mode::List, Command::Open) => Mode::Detail,
            (Mode::List, Command::SearchStart) => Mode::Search,
            (Mode::List, Command::Add) => Mode::AddForm,
            (Mode::List, Command::Edit) => Mode::EditForm,
            (Mode::List, Command::Delete) => Mode::ConfirmDelete,
            (Mode::List, Command::MoveUp) => Mode::List,
            (Mode::List, Command::MoveDown) => Mode::List,
            (Mode::List, Command::PageUp) => Mode::List,
            (Mode::List, Command::PageDown) => Mode::List,
            (Mode::List, Command::Tick) => Mode::List,
            (Mode::List, Command::Quit) => Mode::Exit,
            // From `Detail`
            (Mode::Detail, Command::Back) => Mode::List,
            (Mode::Detail, Command::Tick) => Mode::Detail,
            (Mode::Detail, Command::Quit) => Mode::Exit,
            // From `Search`
            (Mode::Search, Command::SearchInput(_)) => Mode::Search,
            (Mode::Search, Command::SearchApply) => Mode::List,
            (Mode::Search, Command::SearchCancel) => Mode::List,
            (Mode::Search, Command::Tick) => Mode::Search,
            // From `AddForm`
            (Mode::AddForm, Command::Confirm) => Mode::List,
            (Mode::AddForm, Command::Cancel) => Mode::List,
            (Mode::AddForm, Command::Tick) => Mode::AddForm,
            (Mode::AddForm, Command::Quit) => Mode::Exit,
            // From `EditForm`
            (Mode::EditForm, Command::Confirm) => Mode::List,
            (Mode::EditForm, Command::Cancel) => Mode::List,
            (Mode::EditForm, Command::Tick) => Mode::EditForm,
            (Mode::EditForm, Command::Quit) => Mode::Exit,
            // From `ConfirmDelete`
            (Mode::ConfirmDelete, Command::Confirm) => Mode::List,
            (Mode::ConfirmDelete, Command::Cancel) => Mode::List,
            (Mode::ConfirmDelete, Command::Tick) => Mode::ConfirmDelete,
            (Mode::ConfirmDelete, Command::Quit) => Mode::Exit,
            (s, _) => s,
        }
    }

    pub fn update(&mut self, cmd: Command) {
        match (&self.mode, &cmd) {
            // When increase the index, cursor move down;
            // When decrease the index, cursor move up.
            (Mode::List, Command::MoveUp) => {
                self.list_index = self.list_index.saturating_sub(1);
                self.sync_scroll();
            }
            (Mode::List, Command::MoveDown) => {
                let max = self.list_len().saturating_sub(1);
                self.list_index = (self.list_index + 1).min(max);
                self.sync_scroll();
            }
            (Mode::List, Command::PageUp) => {
                let step = self.page_step();
                self.list_index = self.list_index.saturating_sub(step);
                self.sync_scroll();
            }
            (Mode::List, Command::PageDown) => {
                let step = self.page_step();
                let max = self.list_len().saturating_sub(1);
                self.list_index = (self.list_index + step).min(max);
                self.sync_scroll();
            }
            (Mode::List, Command::SearchStart) => {
                self.search_query.clear();
                self.search_cursor = 0;
                self.filter_secrets("");
                self.list_index = 0;
                self.scroll_offset = 0;
            }
            (Mode::List, Command::Add) => {
                self.form = Some(SecretForm::default());
            }
            (Mode::List, Command::Edit) => {
                if self.form.is_none() {
                    self.status = Some("Edit not ready: form not prepared".to_string());
                    // stay in List, do NOT transition
                    return;
                }
            }
            (Mode::List, Command::Delete) => {
                self.pending_delete_id = self.selected_id();
            }
            (Mode::Search, Command::SearchInput(c)) => {
                self.search_query.push(*c);
                self.search_cursor = self.search_query.len();
                let query = self.search_query.clone();
                self.filter_secrets(&query);
                self.list_index = 0;
                self.scroll_offset = 0;
            }
            (Mode::Search, Command::SearchCancel) => {
                self.search_query.clear();
                self.search_cursor = 0;
                self.filtered_indices.clear();
                self.list_index = 0;
                self.scroll_offset = 0;
            }
            (Mode::Search, Command::SearchApply) => {
                let query = self.search_query.clone();
                self.filter_secrets(&query);
                self.list_index = 0;
                self.scroll_offset = 0;
            }
            (Mode::AddForm, Command::Confirm) | (Mode::AddForm, Command::Cancel) => {
                self.form = None;
            }
            (Mode::EditForm, Command::Confirm) | (Mode::EditForm, Command::Cancel) => {
                self.form = None;
            }
            (Mode::ConfirmDelete, Command::Confirm) | (Mode::ConfirmDelete, Command::Cancel) => {
                self.pending_delete_id = None;
            }
            (_, Command::Tick) => {
                self.last_tick = Instant::now();
            }
            _ => {}
        }

        self.mode = Self::transition(self.mode, cmd);
    }

    fn filter_secrets(&mut self, query: &str) {
        self.filtered_indices.clear();

        if query.is_empty() {
            self.filtered_indices.extend(0..self.secrets.len());
            return;
        }

        let q = query.to_lowercase();
        self.filtered_indices.extend(
            self.secrets
                .iter()
                .enumerate()
                .filter(|(_, s)| {
                    s.name.to_lowercase().contains(&q)
                        || s.kind
                            .as_ref()
                            .map(|k| k.to_lowercase().contains(&q))
                            .unwrap_or(false)
                        || s.note
                            .as_ref()
                            .map(|n| n.to_lowercase().contains(&q))
                            .unwrap_or(false)
                })
                .map(|(i, _)| i),
        );
    }

    fn list_len(&self) -> usize {
        if self.filtered_indices.is_empty() {
            self.secrets.len()
        } else {
            self.filtered_indices.len()
        }
    }

    fn page_step(&self) -> usize {
        self.page_size.max(1)
    }

    /// Update the scroll offset
    fn sync_scroll(&mut self) {
        let page = self.page_step();
        if self.list_index < self.scroll_offset {
            self.scroll_offset = self.list_index;
        } else if self.list_index >= self.scroll_offset + page {
            self.scroll_offset = self.list_index + 1 - page;
        }
    }

    fn selected_metadata(&self) -> Option<&SecretMetadata> {
        let index = if self.filtered_indices.is_empty() {
            self.secrets.get(self.list_index).map(|_| self.list_index)
        } else {
            self.filtered_indices.get(self.list_index).copied()
        };

        index.and_then(|i| self.secrets.get(i))
    }

    pub fn selected_id(&self) -> Option<Uuid> {
        self.selected_metadata().map(|m| m.id)
    }
}

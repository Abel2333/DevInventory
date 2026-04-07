//! Pure state transitions for the TUI.
//!
//! Boundary rules:
//! - Only update in-memory `AppState` fields.
//! - No service calls, no database access, no terminal I/O.
//! - All mode changes must happen here via `transition` + `update`.
//!
//! Examples:
//! - `(Mode::List, Command::MoveDown)` updates `list_index` and scroll offsets.
//! - `(Mode::Search, Command::SearchInput('a'))` updates `search_query`
//!   and `filtered_indices`.
//! - `(Mode::List, Command::OpenDetail)` should only change `mode` (after validation),
//!   while `app` handles loading the selected secret.
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
    pub cursor_field: FormField,
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

    fn next_field(&mut self) {
        self.cursor_field = match self.cursor_field {
            FormField::Name => FormField::Kind,
            FormField::Kind => FormField::Note,
            FormField::Note => FormField::Plaintext,
            FormField::Plaintext => FormField::Name,
        }
    }

    fn prev_field(&mut self) {
        self.cursor_field = match self.cursor_field {
            FormField::Name => FormField::Plaintext,
            FormField::Kind => FormField::Name,
            FormField::Note => FormField::Kind,
            FormField::Plaintext => FormField::Note,
        }
    }

    fn push_char(&mut self, c: char) {
        match self.cursor_field {
            FormField::Name => self.name.push(c),
            FormField::Kind => self.kind.get_or_insert(String::new()).push(c),
            FormField::Note => self.note.get_or_insert(String::new()).push(c),
            FormField::Plaintext => self.plaintext.push(c),
        }

        self.dirty = true
    }

    fn backspace(&mut self) {
        match self.cursor_field {
            FormField::Name => {
                self.name.pop();
            }
            FormField::Kind => {
                if let Some(v) = self.kind.as_mut() {
                    v.pop();
                }
            }
            FormField::Note => {
                if let Some(v) = self.note.as_mut() {
                    v.pop();
                }
            }
            FormField::Plaintext => {
                self.plaintext.pop();
            }
        }

        self.dirty = true
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
        let filtered_indices = (0..secrets.len()).collect();
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
            filtered_indices,
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
            (Mode::List, Command::OpenDetail) => Mode::Detail,
            (Mode::List, Command::StartSearch) => Mode::Search,
            (Mode::List, Command::StartAdd) => Mode::AddForm,
            (Mode::List, Command::StartEdit) => Mode::EditForm,
            (Mode::List, Command::StartDelete) => Mode::ConfirmDelete,
            (Mode::List, Command::MoveUp) => Mode::List,
            (Mode::List, Command::MoveDown) => Mode::List,
            (Mode::List, Command::PageUp) => Mode::List,
            (Mode::List, Command::PageDown) => Mode::List,
            (Mode::List, Command::Tick) => Mode::List,
            (Mode::List, Command::Quit) => Mode::Exit,
            // From `Detail`
            (Mode::Detail, Command::BackToList) => Mode::List,
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
            (Mode::List, Command::StartSearch) => {
                self.search_query.clear();
                self.search_cursor = 0;
                self.filter_secrets("");
                self.list_index = 0;
                self.scroll_offset = 0;
            }
            (Mode::List, Command::StartAdd) => {
                self.form = Some(SecretForm::default());
            }
            (Mode::List, Command::StartDelete) => {
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
                self.filter_secrets("");
                self.list_index = 0;
                self.scroll_offset = 0;
            }
            (Mode::Search, Command::SearchApply) => {
                let query = self.search_query.clone();
                self.filter_secrets(&query);
                self.list_index = 0;
                self.scroll_offset = 0;
            }

            // Add Form
            (Mode::AddForm, Command::Confirm) | (Mode::AddForm, Command::Cancel) => {
                self.form = None;
            }
            (Mode::AddForm, Command::FormNextField) => {
                let Some(form) = self.form.as_mut() else {
                    return;
                };

                form.next_field();
            }
            (Mode::AddForm, Command::FormPrevField) => {
                let Some(form) = self.form.as_mut() else {
                    return;
                };

                form.prev_field();
            }
            (Mode::AddForm, Command::FormInput(c)) => {
                let Some(form) = self.form.as_mut() else {
                    return;
                };

                form.push_char(*c);
            }
            (Mode::AddForm, Command::FormBackspace) => {
                let Some(form) = self.form.as_mut() else {
                    return;
                };

                form.backspace();
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

    pub fn rebuild_filter(&mut self) {
        self.filtered_indices.clear();

        if self.search_query.is_empty() {
            self.filtered_indices.extend(0..self.secrets.len());
            return;
        }

        let q = self.search_query.to_lowercase();
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

    pub fn clamp_selection(&mut self) {
        let len = self.filtered_indices.len();

        if len == 0 {
            self.list_index = 0;
        } else {
            self.list_index = self.list_index.min(len - 1)
        }
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
        self.filtered_indices.len()
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

    pub fn normalize_view(&mut self) {
        if self.filtered_indices.is_empty() {
            self.list_index = 0;
            self.scroll_offset = 0;
            return;
        }

        self.clamp_selection();
        self.sync_scroll();
    }

    pub fn reset_search(&mut self) {
        self.search_query.clear();
        self.search_cursor = 0;
        self.rebuild_filter();
        self.list_index = 0;
        self.scroll_offset = 0;
    }
    pub fn refresh_search_results(&mut self) {
        self.search_cursor = self.search_query.len();
        self.rebuild_filter();
        self.list_index = 0;
        self.scroll_offset = 0;
    }

    fn selected_metadata(&self) -> Option<&SecretMetadata> {
        let index = self.filtered_indices.get(self.list_index).copied()?;
        self.secrets.get(index)
    }

    pub fn selected_id(&self) -> Option<Uuid> {
        self.selected_metadata().map(|m| m.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SecretMetadata;
    use chrono::Utc;
    use uuid::Uuid;

    fn metadata(name: &str) -> SecretMetadata {
        SecretMetadata {
            id: Uuid::new_v4(),
            name: name.to_string(),
            kind: Some("test".to_string()),
            note: Some(format!("note-{name}")),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_state() -> AppState {
        AppState::new(vec![metadata("alpha"), metadata("beta"), metadata("gamma")])
    }

    #[test]
    fn move_down() {
        let mut state = sample_state();

        state.update(Command::MoveDown);

        assert_eq!(state.list_index, 1);
        assert_eq!(state.mode, Mode::List);
    }

    #[test]
    fn start_search() {
        let mut state = sample_state();
        state.search_query = "abc".to_string();
        state.search_cursor = 3;
        state.list_index = 2;
        state.scroll_offset = 1;

        state.update(Command::StartSearch);

        assert_eq!(state.mode, Mode::Search);
        assert!(state.search_query.is_empty());
        assert_eq!(state.search_cursor, 0);
        assert_eq!(state.list_index, 0);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn search_input() {
        let mut state = sample_state();
        state.update(Command::StartSearch);

        state.update(Command::SearchInput('b'));

        assert_eq!(state.mode, Mode::Search);
        assert_eq!(state.search_query, "b");
        assert_eq!(state.search_cursor, 1);
        assert_eq!(state.filtered_indices.len(), 1);
        assert_eq!(state.secrets[state.filtered_indices[0]].name, "beta");
    }

    #[test]
    fn confirm_back_to_list() {
        let mut state = sample_state();

        state.update(Command::StartAdd);
        state.update(Command::Confirm);

        assert_eq!(state.mode, Mode::List);
        assert!(state.form.is_none());
    }

    #[test]
    fn delete_item() {
        let mut state = sample_state();

        let expected = state.selected_id();
        state.update(Command::StartDelete);

        assert_eq!(state.mode, Mode::ConfirmDelete);
        assert_eq!(state.pending_delete_id, expected);
    }

    #[test]
    fn tick_updates() {
        let mut state = sample_state();
        let before = state.last_tick;

        std::thread::sleep(std::time::Duration::from_millis(10));
        state.update(Command::Tick);

        assert!(state.last_tick >= before);
    }
}

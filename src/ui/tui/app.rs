//! Side-effect coordinator for the TUI.
//!
//! Boundary rules:
//! - Call `SecretService` and update `AppState` data fields with results.
//! - Do not decide or mutate `mode` directly.
//! - Always delegate transitions to `AppState::update`.
//!
//! Examples:
//! - On `Command::Open`, fetch the selected secret and set `state.current_secret`,
//!   then call `state.update(Command::Open)`.
//! - On `Command::Confirm` in `AddForm`, call `service.add_secret(...)`,
//!   set a success status, then call `state.update(Command::Confirm)`.
use ratatui::prelude::Frame;

use crate::{
    app::SecretService,
    error::AppError,
    ui::tui::{
        commands::Command,
        state::{AppState, Mode, SecretForm},
    },
};

pub struct TuiApp {
    state: AppState,
    service: SecretService,
}

impl TuiApp {
    pub fn new(state: AppState, service: SecretService) -> Self {
        Self { state, service }
    }

    /// Provide read-only access to the current app state.
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Provide mutable access for tests or external coordinators.
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    /// Route a high-level Command into state changes and side effects.
    pub async fn on_command(&mut self, _command: Command) -> Result<(), AppError> {
        // TODO: update AppState based on command and trigger service calls.
        match (self.state.mode, &_command) {
            (Mode::List, Command::Edit) => {
                let Some(id) = self.state.selected_id() else {
                    self.state.status = Some("No item selected".to_string());
                    return Ok(());
                };
                let secret_item = self.service.get_secret(id).await?;
                self.state.form = Some(SecretForm::from_secret(&secret_item));
            }
            (Mode::EditForm, Command::Back) => {
                self.state.form = None;
            }
            (Mode::ConfirmDelete, Command::Confirm) => {
                let Some(id) = self.state.pending_delete_id else {
                    self.state.status = Some("No item selected".to_string());
                    return Ok(());
                };
                self.service.delete_secret(id).await?;
            }
            (Mode::List, Command::Open) => {
                let Some(id) = self.state.selected_id() else {
                    self.state.status = Some("No item selected".to_string());
                    return Ok(());
                };

                self.state.current_secret = Some(self.service.get_secret(id).await?);
            }
            (Mode::AddForm, Command::Confirm) => {
                let Some(form) = self.state.form.as_ref() else {
                    self.state.status = Some("No form data".to_string());
                    return Ok(());
                };
                if form.name.trim().is_empty() {
                    self.state.status = Some("Name cannot be empty".to_string());
                    return Ok(());
                }

                self.service
                    .add_secret(
                        form.name.clone(),
                        form.plaintext.as_bytes().to_vec(),
                        form.kind.clone(),
                        form.note.clone(),
                    )
                    .await?;

                self.state.status = Some("Saved".to_string());
            }
            _ => {}
        }
        self.state.update(_command);
        Ok(())
    }

    /// Advance time-based behaviors (tick) like animations or polling.
    pub fn on_tick(&mut self) {
        // TODO: update tick timestamps and refresh any time-driven state.
        todo!();
    }

    /// Refresh state from the service layer (load list/detail data).
    pub async fn sync_from_service(&mut self) {
        // TODO: call SecretService and merge results into AppState.
        todo!();
    }

    /// Draw the current UI from AppState into the ratatui Frame.
    pub fn draw(&mut self, _frame: &mut Frame) {
        // TODO: render layout and widgets based on state.mode and state data.
        todo!();
    }
}

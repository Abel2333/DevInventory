//! Side-effect coordinator for the TUI.
//!
//! Boundary rules:
//! - Call `SecretService` and update `AppState` data fields with results.
//! - Do not decide or mutate `mode` directly.
//! - Always delegate transitions to `AppState::update`.
//!
//! Examples:
//! - On `Command::OpenDetail`, fetch the selected secret and set `state.current_secret`,
//!   then call `state.update(Command::OpenDetail)`.
//! - On `Command::Confirm` in `AddForm`, call `service.add_secret(...)`,
//!   set a success status, then call `state.update(Command::Confirm)`.
use ratatui::{
    prelude::Frame,
    widgets::{Block, Paragraph},
};

use crate::{
    app::SecretService,
    error::AppError,
    ui::tui::{
        commands::Command,
        events::RawCommand,
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
    pub async fn on_command(&mut self, command: Command) -> Result<(), AppError> {
        match (self.state.mode, &command) {
            (Mode::List, Command::OpenDetail) => {
                let Some(id) = self.state.selected_id() else {
                    self.state.status = Some("No item selected".to_string());
                    return Ok(());
                };
                match self.service.get_secret(id).await {
                    Ok(secret) => {
                        self.state.current_secret = Some(secret);
                    }
                    Err(err) => {
                        self.state.status = Some(err.to_string());
                        return Ok(());
                    }
                }
            }
            (Mode::List, Command::StartEdit) => {
                let Some(id) = self.state.selected_id() else {
                    self.state.status = Some("No item selected".to_string());
                    return Ok(());
                };

                match self.service.get_secret(id).await {
                    Ok(secret) => {
                        self.state.form = Some(SecretForm::from_secret(&secret));
                    }
                    Err(err) => {
                        self.state.status = Some(err.to_string());
                        return Ok(());
                    }
                }
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

                match self
                    .service
                    .add_secret(
                        form.name.clone(),
                        form.plaintext.as_bytes().to_vec(),
                        form.kind.clone(),
                        form.note.clone(),
                    )
                    .await
                {
                    Ok(_) => {
                        self.state.status = Some("Saved".to_string());
                        self.sync_from_service().await?;
                    }
                    Err(err) => {
                        self.state.status = Some(err.to_string());
                        return Ok(());
                    }
                }
            }

            (Mode::EditForm, Command::Confirm) => {
                let Some(form) = self.state.form.as_ref() else {
                    self.state.status = Some("No form data".to_string());
                    return Ok(());
                };

                if form.name.trim().is_empty() {
                    self.state.status = Some("Name cannot be empty".to_string());
                    return Ok(());
                }

                let Some(id) = form.id else {
                    self.state.status = Some("Missing secret id".to_string());
                    return Ok(());
                };

                match self
                    .service
                    .update_secret(
                        id,
                        form.name.clone(),
                        form.plaintext.as_bytes().to_vec(),
                        form.kind.clone(),
                        form.note.clone(),
                    )
                    .await
                {
                    Ok(secret) => {
                        self.state.current_secret = Some(secret);
                        self.state.status = Some("Updated".to_string());
                        self.sync_from_service().await?;
                    }
                    Err(err) => {
                        self.state.status = Some(err.to_string());
                        return Ok(());
                    }
                }
            }
            (Mode::ConfirmDelete, Command::Confirm) => {
                let Some(id) = self.state.pending_delete_id else {
                    self.state.status = Some("No item selected".to_string());
                    return Ok(());
                };

                match self.service.delete_secret(id).await {
                    Ok(()) => {
                        if self
                            .state
                            .current_secret
                            .as_ref()
                            .map(|secret| secret.id == id)
                            .unwrap_or(false)
                        {
                            self.state.current_secret = None;
                        }
                        self.state.status = Some("Deleted".to_string());
                        self.sync_from_service().await?;
                    }
                    Err(err) => {
                        self.state.status = Some(err.to_string());
                        return Ok(());
                    }
                }
            }
            _ => {}
        }
        self.state.update(command);
        Ok(())
    }

    /// Advance time-based behaviors (tick) like animations or polling.
    pub fn on_tick(&mut self) {
        self.state.update(Command::Tick);
    }

    /// Refresh state from the service layer (load list/detail data).
    pub async fn sync_from_service(&mut self) -> Result<(), AppError> {
        self.state.secrets = self.service.list_secrets().await?;
        self.state.rebuild_filter();
        self.state.clamp_selection();
        self.state.normalize_view();
        Ok(())
    }

    /// Draw the current UI from AppState into the ratatui Frame.
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let status = self.state.status.as_deref().unwrap_or("Ready");
        let content = format!(
            "DevInventory\n\nMode   : {:?}\nStatus  :{}\n\nPress q to quit",
            self.state.mode, status
        );
        let block = Block::bordered().title("DevInventory");
        let paragraph = Paragraph::new(content).block(block);

        frame.render_widget(paragraph, area);
    }
}

/// Resolve a mode-agnostic `RawCommand` into a mode-aware application `Command`.
pub fn normalized_command(mode: Mode, raw: RawCommand) -> Command {
    match (mode, raw) {
        (_, RawCommand::Quit) => Command::Quit,
        (_, RawCommand::Tick) => Command::Tick,
        (_, RawCommand::None) => Command::None,

        (_, RawCommand::Up) => Command::MoveUp,
        (_, RawCommand::Down) => Command::MoveDown,
        (_, RawCommand::PageUp) => Command::PageUp,
        (_, RawCommand::PageDown) => Command::PageDown,

        (Mode::List, RawCommand::Primary) => Command::OpenDetail,
        (Mode::Detail, RawCommand::Secondary) => Command::BackToList,

        (Mode::List, RawCommand::Search) => Command::StartSearch,
        (Mode::Search, RawCommand::Primary) => Command::SearchApply,
        (Mode::Search, RawCommand::Secondary) => Command::SearchCancel,
        (Mode::Search, RawCommand::Char(c)) => Command::SearchInput(c),

        (Mode::List, RawCommand::Add) => Command::StartAdd,
        (Mode::List, RawCommand::Edit) => Command::StartEdit,
        (Mode::List, RawCommand::Delete) => Command::StartDelete,

        (Mode::AddForm, RawCommand::Primary) => Command::Confirm,
        (Mode::AddForm, RawCommand::Secondary) => Command::Cancel,
        (Mode::EditForm, RawCommand::Primary) => Command::Confirm,
        (Mode::EditForm, RawCommand::Secondary) => Command::Cancel,
        (Mode::ConfirmDelete, RawCommand::Primary) => Command::Confirm,
        (Mode::ConfirmDelete, RawCommand::Secondary) => Command::Cancel,

        _ => Command::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::tui::events::RawCommand;

    #[test]
    fn normalized_command_maps() {
        let cases = [
            (Mode::List, RawCommand::Primary, Command::OpenDetail),
            (Mode::Detail, RawCommand::Secondary, Command::BackToList),
            (Mode::List, RawCommand::Search, Command::StartSearch),
            (Mode::List, RawCommand::Add, Command::StartAdd),
            (Mode::List, RawCommand::Edit, Command::StartEdit),
            (Mode::List, RawCommand::Delete, Command::StartDelete),
            (Mode::AddForm, RawCommand::Primary, Command::Confirm),
            (Mode::AddForm, RawCommand::Secondary, Command::Cancel),
            (Mode::ConfirmDelete, RawCommand::Secondary, Command::Cancel),
        ];

        for (mode, raw, expeected) in cases {
            assert_eq!(normalized_command(mode, raw), expeected);
        }
    }
}

use ratatui::prelude::Frame;

use crate::{
    app::SecretService,
    ui::tui::{commands::Command, state::AppState},
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
    pub fn on_command(&mut self, _command: Command) {
        // TODO: update AppState based on command and trigger service calls.
        todo!();
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

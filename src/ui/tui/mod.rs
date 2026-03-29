//! TUI module boundaries and responsibilities.
//!
//! This module is intentionally split into three stages to keep responsibilities clear:
//! 1. `events` maps raw terminal input into `Command` values.
//! 2. `state` applies *pure* state transitions (no I/O, no service calls).
//! 3. `app` performs side effects (service calls) and then delegates to `state`.
//!
//! If you change behavior, keep the rules below intact to avoid drift.
//!
//! # Boundaries (with examples)
//!
//! ## `events` (input -> command)
//! - Only maps input events to `Command`.
//! - Does not read or mutate `AppState`.
//! - Does not call services.
//!
//! Example:
//! - `KeyCode::Char('a')` -> `Command::Add`
//! - `KeyCode::Char('/')` -> `Command::SearchStart`
//!
//! ## `state` (command -> state transition)
//! - Pure state updates: mode changes, cursor/index, form buffers, search query.
//! - Validates whether a transition is allowed.
//! - Must not call `SecretService` or touch I/O.
//!
//! Example:
//! - `(Mode::List, Command::Open)` transitions to `Mode::Detail`
//!   only if a selection exists; otherwise set `status` and stay in `List`.
//! - `(Mode::Search, Command::SearchInput('a'))` appends to `search_query`
//!   and updates `filtered_indices`.
//!
//! ## `app` (side effects + state update)
//! - Uses `SecretService` to fetch or mutate data.
//! - May update `AppState` data fields *only* with results of service calls
//!   (e.g., `current_secret`, `secrets`, `status`), but must not decide mode.
//! - Always delegates mode transitions to `AppState::update`.
//!
//! Example:
//! - When command is `Open`, `app` calls `service.get_secret(...)`
//!   and stores it into `state.current_secret`, then calls `state.update(cmd)`.
//! - When command is `Confirm` in `AddForm`, `app` calls `service.add_secret(...)`,
//!   sets a success status, then calls `state.update(cmd)`.
//!
//! Keeping these boundaries prevents inconsistent transitions and makes
//! state changes easier to test.

use std::time::{Duration, Instant};

use crate::{app::SecretService, ui::tui::events::poll_raw_command};
pub mod app;
pub mod commands;
pub mod events;
pub mod state;
pub mod terminal;

pub async fn run_tui(service: SecretService) -> anyhow::Result<()> {
    let mut terminal = terminal::init()?;
    let secrets = service.list_secrets().await?;
    let state = state::AppState::new(secrets);
    let mut app = app::TuiApp::new(state, service);
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| app.draw(frame))?;

        let raw = poll_raw_command(tick_rate, &mut last_tick)?;
        let command = app::normalized_command(app.state().mode, raw);

        match command {
            commands::Command::Tick => app.on_tick(),
            commands::Command::None => {}
            _ => app.on_command(command).await?,
        }

        if app.state().mode == state::Mode::Exit {
            break;
        }
    }

    terminal::restore()?;
    Ok(())
}

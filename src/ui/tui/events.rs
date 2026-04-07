//! Input mapping only: terminal events -> `RawCommand`.
//!
//! Boundary rules:
//! - Do not read or mutate `AppState`.
//! - Do not call services or perform I/O beyond reading terminal events.
//! - Keep mappings deterministic (no hidden state).
//!
//! Example:
//! - `KeyCode::Char('q')` -> `RawCommand::Char('q')`
//! - `KeyCode::Char(c)` -> `RawCommand::Char(c)`
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawCommand {
    Tick,
    None,

    Up,
    Down,
    PageUp,
    PageDown,

    Primary,
    Secondary,

    Char(char),
    Backspace,
}

pub fn map_key(key: KeyEvent) -> RawCommand {
    match (key.code, key.modifiers) {
        (KeyCode::Up, _) => RawCommand::Up,
        (KeyCode::Down, _) => RawCommand::Down,
        (KeyCode::PageUp, _) => RawCommand::PageUp,
        (KeyCode::PageDown, _) => RawCommand::PageDown,
        (KeyCode::Enter, _) => RawCommand::Primary,
        (KeyCode::Esc, _) => RawCommand::Secondary,
        (KeyCode::Char(c), _) => RawCommand::Char(c),
        (KeyCode::Backspace, _) => RawCommand::Backspace,
        _ => RawCommand::None,
    }
}

pub fn poll_raw_command(tick_rate: Duration, last_tick: &mut Instant) -> io::Result<RawCommand> {
    let timeout = tick_rate.saturating_sub(last_tick.elapsed());
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(k) => Ok(map_key(k)),
            _ => Ok(RawCommand::None),
        }
    } else {
        *last_tick = Instant::now();
        Ok(RawCommand::Tick)
    }
}

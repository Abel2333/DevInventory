//! Input mapping only: terminal events -> `RawCommand`.
//!
//! Boundary rules:
//! - Do not read or mutate `AppState`.
//! - Do not call services or perform I/O beyond reading terminal events.
//! - Keep mappings deterministic (no hidden state).
//!
//! Example:
//! - `KeyCode::Char('q')` -> `RawCommand::Quit`
//! - `KeyCode::Char(c)` -> `RawCommand::Char(c)`
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawCommand {
    Quit,
    Tick,
    None,

    Up,
    Down,
    PageUp,
    PageDown,

    Primary,
    Secondary,

    Search,
    Add,
    Edit,
    Delete,

    Char(char),
}

pub fn map_key(key: KeyEvent) -> RawCommand {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => RawCommand::Quit,
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => RawCommand::Up,
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => RawCommand::Down,
        (KeyCode::PageUp, _) => RawCommand::PageUp,
        (KeyCode::PageDown, _) => RawCommand::PageDown,
        (KeyCode::Enter, _) => RawCommand::Primary,
        (KeyCode::Esc, _) => RawCommand::Secondary,
        (KeyCode::Char('a'), _) => RawCommand::Add,
        (KeyCode::Char('e'), _) => RawCommand::Edit,
        (KeyCode::Char('d'), _) => RawCommand::Delete,
        (KeyCode::Char('/'), _) => RawCommand::Search,
        (KeyCode::Char(c), _) => RawCommand::Char(c),
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
        Ok(RawCommand::Tick)
    }
}

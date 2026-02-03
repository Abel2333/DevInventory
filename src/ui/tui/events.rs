use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::ui::tui::{commands::Command};

pub fn map_key(key: KeyEvent) -> Command {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => Command::Quit,
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Command::MoveUp,
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Command::MoveDown,
        (KeyCode::Enter, _) => Command::Open,
        (KeyCode::Esc, _) => Command::Back,
        (KeyCode::Char('a'), _) => Command::Add,
        (KeyCode::Char('e'), _) => Command::Edit,
        (KeyCode::Char('d'), _) => Command::Delete,
        (KeyCode::Char('/'), _) => Command::SearchStart,
        (KeyCode::Char(c), _) => Command::SearchInput(c),
        _ => Command::None,
    }
}

pub fn poll_command(tick_rate: Duration, last_tick: &mut Instant) -> io::Result<Command> {
    let timeout = tick_rate.saturating_sub(last_tick.elapsed());
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(k) => Ok(map_key(k)),
            _ => Ok(Command::None),
        }
    } else {
        Ok(Command::Tick)
    }
}

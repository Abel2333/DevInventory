use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::tui::commands::Command;

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

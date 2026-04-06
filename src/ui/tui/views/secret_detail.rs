use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::tui::state::{AppState, Mode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = match (&state.mode, &state.current_secret) {
        (Mode::Detail, Some(secret)) => format!(
            "Name: {}\nKind: {}\nNote: {}",
            secret.name,
            secret.kind.as_deref().unwrap_or("-"),
            secret.note.as_deref().unwrap_or("-"),
        ),
        (Mode::Search, _) => "Type to search\nEnter apply | Esc cancel".to_string(),
        _ => "Select a secret and press Enter".to_string(),
    };

    let detail =
        Paragraph::new(content).block(Block::default().title("Detail").borders(Borders::ALL));

    frame.render_widget(detail, area);
}

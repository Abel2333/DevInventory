use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::tui::state::{AppState, Mode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = match state.mode {
        Mode::Search => format!("Search: {}", state.search_query),
        _ => format!("{:?}", state.mode),
    };

    let header =
        Paragraph::new(content).block(Block::default().title("Inventory").borders(Borders::ALL));

    frame.render_widget(header, area);
}

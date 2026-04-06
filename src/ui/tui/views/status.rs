use ratatui::{Frame, layout::Rect, widgets::Paragraph};

use crate::ui::tui::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(
        Paragraph::new(format!(
            "Mode: {:?} | Status: {} | q quit | Enter open | / search | a add",
            state.mode,
            state.status.as_deref().unwrap_or("Error")
        )),
        area,
    );
}

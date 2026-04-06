use ratatui::{Frame, layout::Rect};

use crate::ui::tui::{state::AppState, views};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    views::secret_form::render(frame, area, state, "Add Secret")
}

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

use crate::ui::tui::{
    state::AppState,
    views::{secret_detail, secret_list},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Min(0)]).areas(area);

    secret_list::render(frame, list_area, state);
    secret_detail::render(frame, detail_area, state);
}

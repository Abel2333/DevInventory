use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::ui::tui::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let start = state.scroll_offset;
    let end = (start + state.page_size).min(state.filtered_indices.len());

    let items: Vec<ListItem> = state.filtered_indices[start..end]
        .iter()
        .map(|&idx| {
            let secret = &state.secrets[idx];
            let kind = secret.kind.as_deref().unwrap_or("-");
            ListItem::new(format!("{} [{}]", secret.name, kind))
        })
        .collect();

    let mut list_state = ListState::default();

    let list = if items.is_empty() {
        List::new(vec![ListItem::new("No Secrets")])
            .block(Block::default().title("Secrets").borders(Borders::ALL))
    } else {
        let visible_len = end - start;
        let selected = state.list_index.saturating_sub(start).min(visible_len - 1);

        list_state.select(Some(selected));

        List::new(items)
            .block(Block::default().title("Secrets").borders(Borders::ALL))
            .highlight_symbol("> ")
    };

    frame.render_stateful_widget(list, area, &mut list_state);
}

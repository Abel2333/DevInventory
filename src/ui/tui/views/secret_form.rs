use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::tui::state::{AppState, FormField};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, title: &str) {
    let content = match state.form.as_ref() {
        Some(form) => {
            let lines = [
                form_line(form.cursor_field == FormField::Name, "Name", &form.name),
                form_line(
                    form.cursor_field == FormField::Kind,
                    "Kind",
                    form.kind.as_deref().unwrap_or(""),
                ),
                form_line(
                    form.cursor_field == FormField::Note,
                    "Note",
                    form.note.as_deref().unwrap_or(""),
                ),
                form_line(
                    form.cursor_field == FormField::Plaintext,
                    "Plaintext",
                    &form.plaintext,
                ),
            ];

            lines.join("\n")
        }
        None => "Form not initialized".to_string(),
    };

    let form = Paragraph::new(content).block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(form, area);
}

fn form_line(active: bool, label: &str, value: &str) -> String {
    let marker = if active { ">" } else { " " };
    format!("{marker} {:<10}: {}", label, value)
}

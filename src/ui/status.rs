//! Status bar and commands bar rendering.

use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, View};

use super::colors;

/// Draws the commands bar showing available actions for current view.
pub fn draw_commands_bar(frame: &mut Frame, area: Rect, app: &App) {
    let commands = get_commands_for_view(&app.current_view);

    let mut spans: Vec<Span> = vec![Span::styled(" ", Style::default())];

    for (i, (key, desc)) in commands.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(colors::BORDER)));
        }
        spans.push(Span::styled(*key, Style::default().fg(colors::KEY).bold()));
        spans.push(Span::styled(" ", Style::default()));
        spans.push(Span::styled(*desc, Style::default().fg(colors::MUTED)));
    }

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(colors::BORDER));

    let commands_widget = Paragraph::new(Line::from(spans)).block(block);

    frame.render_widget(commands_widget, area);
}

/// Returns the list of commands available for a given view.
fn get_commands_for_view(view: &View) -> Vec<(&'static str, &'static str)> {
    match view {
        View::AuthRequired => vec![("Enter", "authenticate"), ("q", "quit")],
        View::SecretsList => vec![
            ("j/k", "navigate"),
            ("Enter", "view"),
            ("n", "new secret"),
            ("p", "switch project"),
            ("r", "refresh"),
            ("?", "help"),
            ("q", "quit"),
        ],
        View::ProjectSelector => vec![("j/k", "navigate"), ("Enter", "select"), ("Esc", "cancel")],
        View::SecretDetail => vec![
            ("b", "back"),
            ("j/k", "navigate"),
            ("s", "show"),
            ("c", "copy"),
            ("a", "add"),
            ("e/x", "enable/disable"),
            ("p", "project"),
        ],
        View::Input(_) => vec![("Enter", "submit"), ("Esc", "cancel")],
        View::Confirm(_) => vec![("Enter", "confirm"), ("Esc", "cancel")],
    }
}

/// Draws the status bar at the bottom (for messages).
pub fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (text, style) = if let Some(status) = &app.status {
        let style = if status.is_error {
            Style::default().fg(colors::ERROR)
        } else {
            Style::default().fg(colors::SUCCESS)
        };
        (format!(" {} ", status.text), style)
    } else {
        (" Ready".to_string(), Style::default().fg(colors::MUTED))
    };

    let status = Paragraph::new(text).style(style);
    frame.render_widget(status, area);
}

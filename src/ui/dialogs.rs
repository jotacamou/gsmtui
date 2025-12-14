//! Dialog rendering (input, confirm, project selector).

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, ConfirmAction, InputMode};
use crate::constants::dialog;

use super::colors;
use super::utils::centered_rect;

/// Block cursor character for input fields.
pub(crate) const BLOCK_CURSOR: &str = "█";

/// Input field prompt indicator.
pub(crate) const INPUT_INDICATOR: &str = "› ";

/// Draws the text input dialog.
pub fn draw_input_dialog(frame: &mut Frame, mode: &InputMode, app: &App) {
    let (title, prompt, icon) = match mode {
        InputMode::NewSecretName => ("Create New Secret", "Enter a name for your secret:", ""),
        InputMode::NewVersionValue => ("Add New Version", "Enter the secret value:", ""),
    };

    let area = centered_rect(dialog::INPUT_WIDTH, dialog::INPUT_HEIGHT, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::PRIMARY))
        .border_set(symbols::border::DOUBLE)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(icon, Style::default().fg(colors::PRIMARY)),
            Span::styled(" ", Style::default()),
            Span::styled(title, Style::default().fg(Color::White).bold()),
            Span::styled(" ", Style::default()),
        ]))
        .padding(Padding::uniform(1));

    // Build the content
    let content = vec![
        Line::from(""),
        Line::from(Span::styled(prompt, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {INPUT_INDICATOR}"),
                Style::default().fg(colors::MUTED),
            ),
            Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
            Span::styled(
                BLOCK_CURSOR,
                Style::default()
                    .fg(colors::PRIMARY)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Enter", Style::default().fg(colors::KEY).bold()),
            Span::styled(" submit  ", Style::default().fg(colors::MUTED)),
            Span::styled("Esc", Style::default().fg(colors::KEY).bold()),
            Span::styled(" cancel", Style::default().fg(colors::MUTED)),
        ]),
    ];

    let input_widget = Paragraph::new(content).block(block);

    frame.render_widget(input_widget, area);
}

/// Draws the confirmation dialog.
pub fn draw_confirm_dialog(frame: &mut Frame, action: &ConfirmAction) {
    let (title, message, icon) = match action {
        ConfirmAction::DeleteSecret(name) => (
            "Delete Secret",
            format!(
                "Are you sure you want to delete '{name}'?\n\nThis will permanently delete the secret and ALL its versions.\nThis action cannot be undone!"
            ),
            "",
        ),
        ConfirmAction::DestroyVersion(secret, version) => (
            "Destroy Version",
            format!(
                "Are you sure you want to destroy version {version} of '{secret}'?\n\nThe secret data will be permanently destroyed.\nThis action cannot be undone!"
            ),
            "",
        ),
    };

    let area = centered_rect(dialog::CONFIRM_WIDTH, dialog::CONFIRM_HEIGHT, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::ERROR))
        .border_set(symbols::border::DOUBLE)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(icon, Style::default().fg(colors::ERROR)),
            Span::styled(" ", Style::default()),
            Span::styled(title, Style::default().fg(colors::ERROR).bold()),
            Span::styled(" ", Style::default()),
        ]))
        .padding(Padding::uniform(1));

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(&message, Style::default().fg(colors::WARNING))),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Enter", Style::default().fg(colors::ERROR).bold()),
            Span::styled(" confirm deletion  ", Style::default().fg(colors::MUTED)),
            Span::styled("Esc", Style::default().fg(colors::KEY).bold()),
            Span::styled(" cancel", Style::default().fg(colors::MUTED)),
        ]),
    ];

    let confirm_widget = Paragraph::new(content)
        .wrap(Wrap { trim: false })
        .block(block);

    frame.render_widget(confirm_widget, area);
}

/// Draws the project selector dialog.
pub fn draw_project_selector(frame: &mut Frame, app: &App) {
    let area = centered_rect(
        dialog::PROJECT_SELECTOR_WIDTH,
        dialog::PROJECT_SELECTOR_HEIGHT,
        frame.area(),
    );

    // Clear the background
    frame.render_widget(Clear, area);

    // Split area into title bar, list, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // List
            Constraint::Length(3), // Footer with commands
        ])
        .margin(1)
        .split(area);

    // Outer block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::PRIMARY))
        .border_set(symbols::border::DOUBLE)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("", Style::default().fg(colors::ACCENT)),
            Span::styled(" Select Project ", Style::default().fg(Color::White).bold()),
        ]));

    frame.render_widget(block, area);

    // Title/hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Current: ", Style::default().fg(colors::MUTED)),
        Span::styled(
            &app.project_id,
            Style::default().fg(colors::SECONDARY).bold(),
        ),
    ]));
    frame.render_widget(hint, chunks[0]);

    // Build the list of projects
    let items: Vec<ListItem> = app
        .available_projects
        .iter()
        .enumerate()
        .map(|(idx, project)| {
            let is_selected = app.projects_state.selected() == Some(idx);
            let is_current = project.project_id == app.project_id;

            let style = if is_selected {
                Style::default()
                    .bg(colors::SELECTION)
                    .fg(colors::SELECTION_TEXT)
            } else {
                Style::default()
            };

            let number = format!("{:>3}", idx + 1);
            let project_id = project.project_id.clone();
            let display_name = if project.display_name == project.project_id {
                String::new()
            } else {
                format!(" ({})", project.display_name)
            };

            let current_marker = if is_current {
                Span::styled(" (current)", Style::default().fg(colors::SUCCESS))
            } else {
                Span::raw("")
            };

            let content = Line::from(vec![
                Span::styled(number, Style::default().fg(colors::ACCENT)),
                Span::styled("  ", style),
                Span::styled(
                    if is_selected { "▸" } else { " " },
                    Style::default().fg(if is_current {
                        colors::SUCCESS
                    } else {
                        colors::PRIMARY
                    }),
                ),
                Span::styled(" ", style),
                Span::styled(project_id, style.add_modifier(Modifier::BOLD)),
                Span::styled(display_name, style.fg(colors::MUTED)),
                current_marker,
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .highlight_symbol("");

    frame.render_stateful_widget(list, chunks[1], &mut app.projects_state.clone());

    // Footer with commands
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("j/k", Style::default().fg(colors::KEY).bold()),
        Span::styled(" navigate  ", Style::default().fg(colors::MUTED)),
        Span::styled("Enter", Style::default().fg(colors::KEY).bold()),
        Span::styled(" select  ", Style::default().fg(colors::MUTED)),
        Span::styled("Esc", Style::default().fg(colors::KEY).bold()),
        Span::styled(" cancel", Style::default().fg(colors::MUTED)),
    ]));
    frame.render_widget(footer, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_is_visible() {
        assert_eq!(BLOCK_CURSOR, "█");
    }

    #[test]
    fn test_input_indicator_exists() {
        assert_eq!(INPUT_INDICATOR, "> ");
    }
}

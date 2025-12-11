//! List rendering for secrets and versions.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::App;
use crate::secret_client::VersionState;

use super::colors;
use super::empty::draw_empty_state;

/// Draws the list of secrets.
pub fn draw_secrets_list(frame: &mut Frame, area: Rect, app: &App) {
    // Split into header hint and list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Section hint
            Constraint::Min(0),    // List
        ])
        .split(area);

    // Draw section hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("", Style::default().fg(colors::WARNING)),
        Span::styled(" ", Style::default()),
        Span::styled("Secrets", Style::default().fg(colors::PRIMARY).bold()),
        Span::styled(
            " - Select a secret to view versions and values",
            Style::default().fg(colors::MUTED),
        ),
    ]));
    frame.render_widget(hint, chunks[0]);

    // Handle empty state
    if app.secrets.is_empty() {
        draw_empty_state(
            frame,
            chunks[1],
            "No secrets found",
            "Press 'n' to create your first secret",
            "",
        );
        return;
    }

    // Create list items from secrets
    let items: Vec<ListItem> = app
        .secrets
        .iter()
        .enumerate()
        .map(|(idx, secret)| {
            let is_selected = app.secrets_state.selected() == Some(idx);

            let number = format!("{:>3}", idx + 1);
            let name = secret.short_name.clone();
            let date = secret.create_time.clone();

            let style = if is_selected {
                Style::default()
                    .bg(colors::SELECTION)
                    .fg(colors::SELECTION_TEXT)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled(number, Style::default().fg(colors::ACCENT)),
                Span::styled("  ", style),
                Span::styled(
                    "",
                    if is_selected {
                        Style::default().fg(colors::WARNING)
                    } else {
                        Style::default().fg(colors::PRIMARY)
                    },
                ),
                Span::styled(" ", style),
                Span::styled(name, style.add_modifier(Modifier::BOLD)),
                Span::styled("  ", style),
                Span::styled(
                    date,
                    style.fg(if is_selected {
                        colors::SELECTION_TEXT
                    } else {
                        colors::MUTED
                    }),
                ),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    // Create the list widget
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::BORDER))
                .border_set(symbols::border::ROUNDED)
                .title(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(
                        format!("{}", app.secrets.len()),
                        Style::default().fg(colors::SECONDARY).bold(),
                    ),
                    Span::styled(" secrets ", Style::default().fg(Color::White)),
                ]))
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(Style::default()) // We handle highlighting in items
        .highlight_symbol("");

    frame.render_stateful_widget(list, chunks[1], &mut app.secrets_state.clone());
}

/// Draws the versions list.
pub fn draw_versions_list(frame: &mut Frame, area: Rect, app: &App) {
    // Handle empty state
    if app.versions.is_empty() {
        draw_empty_state(
            frame,
            area,
            "No versions yet",
            "Press 'a' to add the first version",
            "A secret needs at least one version to store a value",
        );
        return;
    }

    let items: Vec<ListItem> = app
        .versions
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let is_selected = app.versions_state.selected() == Some(idx);

            let (state_icon, state_color) = match v.state {
                VersionState::Enabled => ("", colors::SUCCESS),
                VersionState::Disabled => ("", colors::WARNING),
                VersionState::Destroyed => ("", colors::ERROR),
                VersionState::Unknown => ("?", colors::MUTED),
            };

            let base_style = if is_selected {
                Style::default()
                    .bg(colors::SELECTION)
                    .fg(colors::SELECTION_TEXT)
            } else {
                Style::default()
            };

            let version_str = format!("v{:<4}", v.version);
            let state_str = v.state.to_string();
            let create_time = v.create_time.clone();

            let mut spans = vec![
                Span::styled(if is_selected { "  " } else { "   " }, base_style),
                Span::styled(state_icon, Style::default().fg(state_color)),
                Span::styled(" ", base_style),
                Span::styled(version_str, base_style.add_modifier(Modifier::BOLD)),
                Span::styled("  ", base_style),
                Span::styled(
                    format!("{state_str:<10}"),
                    base_style.fg(if is_selected {
                        colors::SELECTION_TEXT
                    } else {
                        state_color
                    }),
                ),
                Span::styled("  ", base_style),
                Span::styled(
                    format!("created {create_time}"),
                    base_style.fg(if is_selected {
                        colors::SELECTION_TEXT
                    } else {
                        colors::MUTED
                    }),
                ),
            ];

            // Add destroy time if destroyed
            if let Some(destroy_time) = &v.destroy_time {
                spans.push(Span::styled(
                    "  destroyed ",
                    base_style.fg(if is_selected {
                        colors::SELECTION_TEXT
                    } else {
                        colors::ERROR
                    }),
                ));
                spans.push(Span::styled(
                    destroy_time,
                    base_style.fg(if is_selected {
                        colors::SELECTION_TEXT
                    } else {
                        colors::MUTED
                    }),
                ));
            }

            // Add scheduled destroy time if pending destruction
            if let Some(scheduled) = &v.scheduled_destroy_time {
                spans.push(Span::styled("  ", base_style));
                spans.push(Span::styled("", Style::default().fg(colors::WARNING)));
                spans.push(Span::styled(
                    format!(" destroys {scheduled}"),
                    base_style.fg(if is_selected {
                        colors::SELECTION_TEXT
                    } else {
                        colors::WARNING
                    }),
                ));
            }

            // Add checksum indicator
            if v.has_checksum {
                spans.push(Span::styled("  ", base_style));
                spans.push(Span::styled("", Style::default().fg(colors::SECONDARY)));
            }

            let content = Line::from(spans);

            ListItem::new(content).style(base_style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::BORDER))
                .border_set(symbols::border::ROUNDED)
                .title(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(
                        format!("{}", app.versions.len()),
                        Style::default().fg(colors::SECONDARY).bold(),
                    ),
                    Span::styled(" versions ", Style::default().fg(Color::White)),
                ]))
                .padding(Padding::horizontal(1)),
        )
        .highlight_symbol("");

    frame.render_stateful_widget(list, area, &mut app.versions_state.clone());
}

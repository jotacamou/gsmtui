//! Secret detail view rendering.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::secret_client::ReplicationPolicy;

use super::colors;
use super::lists::draw_versions_list;

/// Draws the secret detail view with versions.
pub fn draw_secret_detail(frame: &mut Frame, area: Rect, app: &App) {
    let Some(secret) = &app.current_secret else {
        return;
    };

    // Calculate info card height dynamically based on content
    let mut extra_rows = 0;
    if !secret.labels.is_empty() {
        extra_rows += 1;
    }
    if !secret.annotations.is_empty() {
        extra_rows += 1;
    }
    if !secret.topics.is_empty() {
        extra_rows += 1;
    }
    if !secret.version_aliases.is_empty() {
        extra_rows += 1;
    }
    if secret.rotation.is_some() {
        extra_rows += 1;
    }
    if secret.version_destroy_ttl.is_some() {
        extra_rows += 1;
    }
    let info_card_height = 5 + extra_rows; // Base: name, created, replication + borders

    // Split the area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),                // Back hint
            Constraint::Length(info_card_height), // Secret info card
            Constraint::Length(2),                // Versions header
            Constraint::Min(0),                   // Versions list / value display
        ])
        .split(area);

    // Draw back hint
    let back_hint = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("", Style::default().fg(colors::PRIMARY)),
        Span::styled(" ", Style::default()),
        Span::styled("Esc", Style::default().fg(colors::KEY).bold()),
        Span::styled(" to go back", Style::default().fg(colors::MUTED)),
    ]));
    frame.render_widget(back_hint, chunks[0]);

    // Draw secret info card
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::PRIMARY))
        .border_set(symbols::border::ROUNDED)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("", Style::default().fg(colors::PRIMARY)),
            Span::styled(
                " Secret Details ",
                Style::default().fg(Color::White).bold(),
            ),
        ]));

    // Replication display
    let replication_str = match &secret.replication {
        ReplicationPolicy::Automatic => "Automatic".to_string(),
        ReplicationPolicy::UserManaged(locations) => {
            if locations.is_empty() {
                "User-managed".to_string()
            } else {
                format!("User-managed ({})", locations.join(", "))
            }
        }
    };

    let mut info_content = vec![
        Line::from(vec![
            Span::styled("  Name        ", Style::default().fg(colors::MUTED)),
            Span::styled(
                &secret.short_name,
                Style::default().fg(Color::White).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Created     ", Style::default().fg(colors::MUTED)),
            Span::styled(&secret.create_time, Style::default().fg(Color::White)),
            Span::styled("    Replication  ", Style::default().fg(colors::MUTED)),
            Span::styled(&replication_str, Style::default().fg(colors::SECONDARY)),
        ]),
    ];

    // Add labels row if any exist
    if !secret.labels.is_empty() {
        let mut label_spans = vec![Span::styled(
            "  Labels      ",
            Style::default().fg(colors::MUTED),
        )];
        for (i, (key, value)) in secret.labels.iter().enumerate() {
            if i > 0 {
                label_spans.push(Span::styled("  ", Style::default()));
            }
            label_spans.push(Span::styled(key, Style::default().fg(colors::ACCENT)));
            label_spans.push(Span::styled("=", Style::default().fg(colors::MUTED)));
            label_spans.push(Span::styled(value, Style::default().fg(Color::White)));
        }
        info_content.push(Line::from(label_spans));
    }

    // Add annotations row if any exist
    if !secret.annotations.is_empty() {
        let mut spans = vec![Span::styled(
            "  Annotations ",
            Style::default().fg(colors::MUTED),
        )];
        for (i, (key, value)) in secret.annotations.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ", Style::default()));
            }
            spans.push(Span::styled(key, Style::default().fg(colors::WARNING)));
            spans.push(Span::styled("=", Style::default().fg(colors::MUTED)));
            spans.push(Span::styled(value, Style::default().fg(Color::White)));
        }
        info_content.push(Line::from(spans));
    }

    // Add topics row if any exist
    if !secret.topics.is_empty() {
        let topics_str = secret.topics.join(", ");
        info_content.push(Line::from(vec![
            Span::styled("  Pub/Sub     ", Style::default().fg(colors::MUTED)),
            Span::styled(topics_str, Style::default().fg(Color::White)),
        ]));
    }

    // Add version aliases if any exist
    if !secret.version_aliases.is_empty() {
        let mut spans = vec![Span::styled(
            "  Aliases     ",
            Style::default().fg(colors::MUTED),
        )];
        for (i, (alias, version)) in secret.version_aliases.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ", Style::default()));
            }
            spans.push(Span::styled(alias, Style::default().fg(colors::KEY)));
            spans.push(Span::styled("→v", Style::default().fg(colors::MUTED)));
            spans.push(Span::styled(
                version.to_string(),
                Style::default().fg(Color::White),
            ));
        }
        info_content.push(Line::from(spans));
    }

    // Add rotation config if present
    if let Some(rotation) = &secret.rotation {
        let mut spans = vec![Span::styled(
            "  Rotation    ",
            Style::default().fg(colors::MUTED),
        )];
        if let Some(period) = &rotation.rotation_period {
            spans.push(Span::styled("every ", Style::default().fg(Color::White)));
            spans.push(Span::styled(period, Style::default().fg(colors::SECONDARY)));
        }
        if let Some(next) = &rotation.next_rotation_time {
            spans.push(Span::styled("  next: ", Style::default().fg(colors::MUTED)));
            spans.push(Span::styled(next, Style::default().fg(Color::White)));
        }
        info_content.push(Line::from(spans));
    }

    // Add version destroy TTL if set
    if let Some(ttl) = &secret.version_destroy_ttl {
        info_content.push(Line::from(vec![
            Span::styled("  Destroy TTL ", Style::default().fg(colors::MUTED)),
            Span::styled(ttl, Style::default().fg(colors::WARNING)),
            Span::styled(
                " (delayed destruction)",
                Style::default().fg(colors::MUTED),
            ),
        ]));
    }

    let info = Paragraph::new(info_content).block(info_block);
    frame.render_widget(info, chunks[1]);

    // Draw versions header with action hints
    let versions_hint = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("", Style::default().fg(colors::ACCENT)),
        Span::styled(" ", Style::default()),
        Span::styled("Versions", Style::default().fg(colors::PRIMARY).bold()),
        Span::styled(" - ", Style::default().fg(colors::MUTED)),
        Span::styled("s", Style::default().fg(colors::KEY).bold()),
        Span::styled(" show  ", Style::default().fg(colors::MUTED)),
        Span::styled("c", Style::default().fg(colors::KEY).bold()),
        Span::styled(" copy  ", Style::default().fg(colors::MUTED)),
        Span::styled("a", Style::default().fg(colors::KEY).bold()),
        Span::styled(" add new", Style::default().fg(colors::MUTED)),
    ]));
    frame.render_widget(versions_hint, chunks[2]);

    // Determine if we're showing the secret value
    let (versions_area, value_area) = if app.revealed_value.is_some() {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(7)])
            .split(chunks[3]);
        (split[0], Some(split[1]))
    } else {
        (chunks[3], None)
    };

    // Draw the versions list
    draw_versions_list(frame, versions_area, app);

    // Draw the revealed value if present
    if let (Some(area), Some(value)) = (value_area, &app.revealed_value) {
        draw_secret_value(frame, area, value);
    }
}

/// Draws the revealed secret value panel.
pub fn draw_secret_value(frame: &mut Frame, area: Rect, value: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::WARNING))
        .border_set(symbols::border::ROUNDED)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("", Style::default().fg(colors::WARNING)),
            Span::styled(
                " Secret Value ",
                Style::default().fg(colors::WARNING).bold(),
            ),
            Span::styled("- press ", Style::default().fg(colors::MUTED)),
            Span::styled("s", Style::default().fg(colors::KEY).bold()),
            Span::styled(" to hide ", Style::default().fg(colors::MUTED)),
        ]))
        .padding(Padding::horizontal(1));

    let content = Paragraph::new(value)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .block(block);

    frame.render_widget(content, area);
}

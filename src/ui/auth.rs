//! Authentication required screen rendering.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::colors;

/// Draws the authentication required screen.
pub fn draw_auth_required(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::WARNING))
        .border_set(symbols::border::ROUNDED)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("", Style::default().fg(colors::WARNING)),
            Span::styled(
                " Authentication Required ",
                Style::default().fg(colors::WARNING).bold(),
            ),
        ]));

    let content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("", Style::default().fg(colors::WARNING))),
        Line::from(""),
        Line::from(Span::styled(
            "GCP credentials not found",
            Style::default().fg(colors::PRIMARY).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "To use this app, you need to authenticate with Google Cloud.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(colors::MUTED)),
            Span::styled("Enter", Style::default().fg(colors::KEY).bold()),
            Span::styled(" to run: ", Style::default().fg(colors::MUTED)),
            Span::styled(
                "gcloud auth application-default login",
                Style::default().fg(colors::SECONDARY),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "This will open your browser to authenticate.",
            Style::default().fg(colors::MUTED),
        )),
        Line::from(Span::styled(
            "Make sure to check all permission boxes in the consent screen.",
            Style::default().fg(colors::WARNING),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(colors::MUTED)),
            Span::styled("q", Style::default().fg(colors::KEY).bold()),
            Span::styled(" to quit", Style::default().fg(colors::MUTED)),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

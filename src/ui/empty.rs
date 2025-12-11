//! Empty state rendering.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::colors;

/// Draws an empty state with icon, title, and description.
pub fn draw_empty_state(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    action: &str,
    description: &str,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::BORDER))
        .border_set(symbols::border::ROUNDED);

    let content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("", Style::default().fg(colors::ACCENT))),
        Line::from(""),
        Line::from(Span::styled(
            title,
            Style::default().fg(colors::PRIMARY).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(action, Style::default().fg(colors::SUCCESS))),
        Line::from(""),
        Line::from(Span::styled(
            description,
            Style::default().fg(colors::MUTED),
        )),
    ];

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

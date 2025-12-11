//! Help overlay rendering.

use ratatui::{
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::constants::dialog;

use super::colors;
use super::utils::centered_rect;

/// Draws a help overlay popup.
pub fn draw_help_overlay(frame: &mut Frame) {
    let area = centered_rect(dialog::HELP_WIDTH, dialog::HELP_HEIGHT, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let help_text = get_help_text();

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::PRIMARY))
                .border_set(symbols::border::DOUBLE)
                .title(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(" Help ", Style::default().fg(Color::White).bold()),
                    Span::styled(
                        "- Press any key to close ",
                        Style::default().fg(colors::MUTED),
                    ),
                ]))
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help, area);
}

/// Returns the help text content.
fn get_help_text() -> Text<'static> {
    let key_style = Style::default().fg(colors::KEY).bold();
    let desc_style = Style::default().fg(Color::White);
    let section_style = Style::default().fg(colors::PRIMARY).bold();

    Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("", section_style),
            Span::styled(" NAVIGATION", section_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("j  ", key_style),
            Span::styled("or ", Style::default().fg(colors::MUTED)),
            Span::styled("Down  ", key_style),
            Span::styled("Move to next item", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("k  ", key_style),
            Span::styled("or ", Style::default().fg(colors::MUTED)),
            Span::styled("Up    ", key_style),
            Span::styled("Move to previous item", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("g  ", key_style),
            Span::styled("or ", Style::default().fg(colors::MUTED)),
            Span::styled("Home  ", key_style),
            Span::styled("Jump to first item", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("G  ", key_style),
            Span::styled("or ", Style::default().fg(colors::MUTED)),
            Span::styled("End   ", key_style),
            Span::styled("Jump to last item", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("Enter     ", key_style),
            Span::styled("Select / View details", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("Esc       ", key_style),
            Span::styled("Go back to previous view", desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("", section_style),
            Span::styled(" SECRETS", section_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("n         ", key_style),
            Span::styled("Create a new secret", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("d         ", key_style),
            Span::styled("Delete selected secret", desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("", section_style),
            Span::styled(" VERSIONS", section_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("a         ", key_style),
            Span::styled("Add a new version with value", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("s         ", key_style),
            Span::styled("Show / Hide secret value", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("c         ", key_style),
            Span::styled("Copy secret value to clipboard", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("e         ", key_style),
            Span::styled("Enable a disabled version", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("x         ", key_style),
            Span::styled("Disable an enabled version", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("d         ", key_style),
            Span::styled("Destroy selected version", desc_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("", section_style),
            Span::styled(" GENERAL", section_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("r         ", key_style),
            Span::styled("Refresh current view", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("?  ", key_style),
            Span::styled("or ", Style::default().fg(colors::MUTED)),
            Span::styled("F1    ", key_style),
            Span::styled("Show this help", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("q  ", key_style),
            Span::styled("or ", Style::default().fg(colors::MUTED)),
            Span::styled("Ctrl+C", key_style),
            Span::styled(" Quit application", desc_style),
        ]),
        Line::from(""),
    ])
}

//! Header rendering with ASCII art logo.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

use super::colors;

/// Returns a randomly selected logo color (selected once at startup).
fn logo_color() -> Color {
    use std::sync::OnceLock;
    static COLOR: OnceLock<Color> = OnceLock::new();
    *COLOR.get_or_init(|| {
        const COLORS: [Color; 4] = [
            Color::Rgb(56, 189, 248),  // Cyan
            Color::Rgb(244, 114, 182), // Pink
            Color::Rgb(192, 132, 252), // Purple
            Color::Rgb(52, 211, 153),  // Emerald
        ];
        COLORS[std::process::id() as usize % COLORS.len()]
    })
}

/// Draws the header with ASCII art logo and subtitle.
pub fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = Style::default().fg(colors::BORDER);
    let dim_style = Style::default().fg(Color::Rgb(55, 65, 81));
    let muted_style = Style::default().fg(Color::Rgb(75, 85, 99));
    let logo_style = Style::default().fg(logo_color()).bold();

    // Status indicator
    let status = if app.is_loading {
        vec![
            Span::styled("┃", border_style),
            Span::styled(
                " ◈ ",
                Style::default()
                    .fg(colors::WARNING)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
            Span::styled("SYNC", Style::default().fg(colors::WARNING).bold()),
            Span::styled(" ┃", border_style),
        ]
    } else {
        vec![
            Span::styled("┃", border_style),
            Span::styled(" ◈ ", Style::default().fg(colors::SUCCESS)),
            Span::styled("Google Cloud", Style::default().fg(colors::SUCCESS).bold()),
            Span::styled(" ┃", border_style),
        ]
    };

    // Top border with status indicator
    let line0 = Line::from(vec![
        Span::styled("┏", Style::default().fg(colors::ACCENT)),
        Span::styled("━━━━━━━━━━━━━━━━━━━━━━━", border_style),
        Span::styled("┓", Style::default().fg(colors::PRIMARY)),
        Span::styled("░▒▓", dim_style),
        status[0].clone(),
        status[1].clone(),
        status[2].clone(),
        status[3].clone(),
        Span::styled("▓▒░", dim_style),
        Span::styled("╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍", dim_style),
    ]);

    // Logo line 1 + info panel top
    let line1 = Line::from(vec![
        Span::styled("┃", Style::default().fg(colors::ACCENT)),
        Span::styled(" ▄████ ▄█▀▀▀ ███▄███▄  ", logo_style),
        Span::styled("┃", Style::default().fg(colors::PRIMARY)),
        Span::styled("  ╭───────────────────────────────╮", border_style),
    ]);

    // Logo line 2 + SECRET::MANAGER title
    let line2 = Line::from(vec![
        Span::styled("┃", Style::default().fg(colors::ACCENT)),
        Span::styled(" ██ ██ ▀███▄ ██ ██ ██  ", logo_style),
        Span::styled("┃", Style::default().fg(colors::PRIMARY)),
        Span::styled("  │ ", border_style),
        Span::styled("◆", Style::default().fg(colors::ACCENT)),
        Span::styled(" SECRET", Style::default().fg(colors::PRIMARY).bold()),
        Span::styled("::", muted_style),
        Span::styled("MANAGER", Style::default().fg(colors::KEY).bold()),
        Span::styled(" ▸▸ ", muted_style),
        Span::styled("TUI", Style::default().fg(colors::ACCENT).bold()),
        Span::styled(" ◆    │", border_style),
    ]);

    // Logo line 3 + info tags
    let line3 = Line::from(vec![
        Span::styled("┃", Style::default().fg(colors::ACCENT)),
        Span::styled(" ▀████ ▄▄▄█▀ ██ ██ ██  ", logo_style),
        Span::styled("┃", Style::default().fg(colors::PRIMARY)),
        Span::styled("  │ ", border_style),
        Span::styled("▪", Style::default().fg(colors::SECONDARY)),
        Span::styled(" GCP  ", Style::default().fg(colors::MUTED)),
        Span::styled("│", dim_style),
        Span::styled(" ▪", Style::default().fg(colors::SUCCESS)),
        Span::styled(" SECRETS ", Style::default().fg(colors::MUTED)),
        Span::styled("│", dim_style),
        Span::styled(" ▪", Style::default().fg(colors::WARNING)),
        Span::styled(format!(" v{} │", env!("CARGO_PKG_VERSION")), border_style),
    ]);

    // Logo line 4 (G tail) + info panel bottom
    let line4 = Line::from(vec![
        Span::styled("┃", Style::default().fg(colors::ACCENT)),
        Span::styled("    ██                 ", logo_style),
        Span::styled("┃", Style::default().fg(colors::PRIMARY)),
        Span::styled("  ╰───────────────────────────────╯", border_style),
    ]);

    // Logo line 5 (G base) + project info
    let line5 = Line::from(vec![
        Span::styled("┗", Style::default().fg(colors::ACCENT)),
        Span::styled("━━▀▀▀", logo_style),
        Span::styled("━━━━━━━━━━━━━━━━━━", border_style),
        Span::styled("┛", Style::default().fg(colors::PRIMARY)),
        Span::styled("  ╾╢", border_style),
        Span::styled(" ⬢  ", Style::default().fg(colors::SECONDARY)),
        Span::styled(
            &app.project_id,
            Style::default().fg(colors::SECONDARY).bold(),
        ),
        Span::styled(" ╟╼", border_style),
    ]);

    let header = Paragraph::new(vec![line0, line1, line2, line3, line4, line5]);
    frame.render_widget(header, area);
}

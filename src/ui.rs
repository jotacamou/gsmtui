//! UI rendering module.
//!
//! This module handles all the terminal UI rendering using Ratatui.
//! Each view is rendered by a separate function for clarity.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, ConfirmAction, InputMode, View};
use crate::secret_client::{ReplicationPolicy, VersionState};

// ============================================================================
// Color Theme - Vibrant colors throughout the app
// ============================================================================

/// Primary accent color (used for titles, highlights)
const COLOR_PRIMARY: Color = Color::Rgb(56, 189, 248);   // Bright cyan
/// Secondary accent color (used for active elements)
const COLOR_SECONDARY: Color = Color::Rgb(52, 211, 153); // Bright emerald
/// Background for selected items
const COLOR_SELECTION: Color = Color::Rgb(99, 102, 241); // Indigo
/// Text on selection
const COLOR_SELECTION_TEXT: Color = Color::White;
/// Muted text color
const COLOR_MUTED: Color = Color::Rgb(148, 163, 184);    // Brighter gray
/// Error/danger color
const COLOR_ERROR: Color = Color::Rgb(251, 113, 133);    // Bright rose
/// Warning color
const COLOR_WARNING: Color = Color::Rgb(251, 191, 36);   // Bright amber
/// Success color
const COLOR_SUCCESS: Color = Color::Rgb(74, 222, 128);   // Bright green
/// Border color
const COLOR_BORDER: Color = Color::Rgb(129, 140, 248);   // Light indigo
/// Key highlight color (for keyboard shortcuts)
const COLOR_KEY: Color = Color::Rgb(244, 114, 182);      // Bright pink
/// Accent color for icons and decorations
const COLOR_ACCENT: Color = Color::Rgb(192, 132, 252);   // Bright purple

// ============================================================================
// Main Draw Function
// ============================================================================

/// Main draw function - dispatches to specific view renderers.
pub fn draw(frame: &mut Frame, app: &App) {
    // Create the main layout: header, content, commands bar, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Header (ASCII art + info panel)
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Commands bar
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    // Draw the header
    draw_header(frame, chunks[0], app);

    // Draw the main content based on current view
    match &app.current_view {
        View::AuthRequired => draw_auth_required(frame, chunks[1]),
        View::SecretsList => draw_secrets_list(frame, chunks[1], app),
        View::SecretDetail => draw_secret_detail(frame, chunks[1], app),
        View::Input(mode) => {
            // Draw the underlying view first
            if let Some(View::SecretsList) = &app.previous_view {
                draw_secrets_list(frame, chunks[1], app);
            } else {
                draw_secret_detail(frame, chunks[1], app);
            }
            // Then draw the input dialog on top
            draw_input_dialog(frame, mode, app);
        }
        View::Confirm(action) => {
            // Draw the underlying view first
            if let Some(View::SecretsList) = &app.previous_view {
                draw_secrets_list(frame, chunks[1], app);
            } else {
                draw_secret_detail(frame, chunks[1], app);
            }
            // Then draw the confirmation dialog on top
            draw_confirm_dialog(frame, action);
        }
        View::ProjectSelector => {
            // Draw the secrets list in the background
            draw_secrets_list(frame, chunks[1], app);
            // Then draw the project selector dialog on top
            draw_project_selector(frame, app);
        }
    }

    // Draw the commands bar (shows available actions)
    draw_commands_bar(frame, chunks[2], app);

    // Draw the status bar (shows messages)
    draw_status_bar(frame, chunks[3], app);

    // Draw help overlay if enabled
    if app.show_help {
        draw_help_overlay(frame);
    }
}

// ============================================================================
// Header - ASCII Art with Gradient
// ============================================================================

// Cyberpunk colors for the ASCII art gradient
const LOGO_COLORS: [Color; 4] = [
    Color::Rgb(56, 189, 248),   // Cyan (matches COLOR_PRIMARY)
    Color::Rgb(244, 114, 182),  // Pink (matches COLOR_KEY)
    Color::Rgb(192, 132, 252),  // Purple (matches COLOR_ACCENT)
    Color::Rgb(52, 211, 153),   // Emerald (matches COLOR_SECONDARY)
];

/// Creates a line with colored characters.
fn colored_line(text: &str, offset: usize) -> Line<'static> {
    let spans: Vec<Span> = text
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let color = LOGO_COLORS[(i + offset) % LOGO_COLORS.len()];
            Span::styled(c.to_string(), Style::default().fg(color).bold())
        })
        .collect();
    Line::from(spans)
}

/// Draws the header with ASCII art logo and subtitle.
fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = Style::default().fg(COLOR_BORDER);
    let dim_style = Style::default().fg(Color::Rgb(55, 65, 81));
    let muted_style = Style::default().fg(Color::Rgb(75, 85, 99));

    // Status indicator
    let status = if app.is_loading {
        vec![
            Span::styled("┃", border_style),
            Span::styled(" ◈ ", Style::default().fg(COLOR_WARNING).add_modifier(Modifier::SLOW_BLINK)),
            Span::styled("SYNC", Style::default().fg(COLOR_WARNING).bold()),
            Span::styled(" ┃", border_style),
        ]
    } else {
        vec![
            Span::styled("┃", border_style),
            Span::styled(" ◈ ", Style::default().fg(COLOR_SUCCESS)),
            Span::styled("Google Cloud", Style::default().fg(COLOR_SUCCESS).bold()),
            Span::styled(" ┃", border_style),
        ]
    };

    // Top border with status indicator
    let line0 = Line::from(vec![
        Span::styled("┏", Style::default().fg(COLOR_ACCENT)),
        Span::styled("━━━━━━━━━━━━━━━━━━━━━━━", border_style),
        Span::styled("┓", Style::default().fg(COLOR_PRIMARY)),
        Span::styled("░▒▓", dim_style),
        status[0].clone(),
        status[1].clone(),
        status[2].clone(),
        status[3].clone(),
        Span::styled("▓▒░", dim_style),
        Span::styled("╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍╍", dim_style),
    ]);

    // Logo line 1 + info panel top
    let mut line1 = Line::from(vec![
        Span::styled("┃", Style::default().fg(COLOR_ACCENT)),
        Span::styled(" ", Style::default()),
    ]);
    line1.spans.extend(colored_line("▄████ ▄█▀▀▀ ███▄███▄", 0).spans);
    line1.spans.extend(vec![
        Span::styled("  ", Style::default()),
        Span::styled("┃", Style::default().fg(COLOR_PRIMARY)),
        Span::styled("  ╭───────────────────────────────╮", border_style),
    ]);

    // Logo line 2 + SECRET::MANAGER title
    let mut line2 = Line::from(vec![
        Span::styled("┃", Style::default().fg(COLOR_ACCENT)),
        Span::styled(" ", Style::default()),
    ]);
    line2.spans.extend(colored_line("██ ██ ▀███▄ ██ ██ ██", 0).spans);
    line2.spans.extend(vec![
        Span::styled("  ", Style::default()),
        Span::styled("┃", Style::default().fg(COLOR_PRIMARY)),
        Span::styled("  │ ", border_style),
        Span::styled("◆", Style::default().fg(COLOR_ACCENT)),
        Span::styled(" SECRET", Style::default().fg(COLOR_PRIMARY).bold()),
        Span::styled("::", muted_style),
        Span::styled("MANAGER", Style::default().fg(COLOR_KEY).bold()),
        Span::styled(" ▸▸ ", muted_style),
        Span::styled("TUI", Style::default().fg(COLOR_ACCENT).bold()),
        Span::styled(" ◆    │", border_style),
    ]);

    // Logo line 3 + info tags
    let mut line3 = Line::from(vec![
        Span::styled("┃", Style::default().fg(COLOR_ACCENT)),
        Span::styled(" ", Style::default()),
    ]);
    line3.spans.extend(colored_line("▀████ ▄▄▄█▀ ██ ██ ██", 0).spans);
    line3.spans.extend(vec![
        Span::styled("  ", Style::default()),
        Span::styled("┃", Style::default().fg(COLOR_PRIMARY)),
        Span::styled("  │ ", border_style),
        Span::styled("▪", Style::default().fg(COLOR_SECONDARY)),
        Span::styled(" GCP  ", Style::default().fg(COLOR_MUTED)),
        Span::styled("│", dim_style),
        Span::styled(" ▪", Style::default().fg(COLOR_SUCCESS)),
        Span::styled(" SECRETS ", Style::default().fg(COLOR_MUTED)),
        Span::styled("│", dim_style),
        Span::styled(" ▪", Style::default().fg(COLOR_WARNING)),
        Span::styled(" v1.0   │", border_style),
    ]);

    // Logo line 4 (G tail) + info panel bottom
    let line4 = Line::from(vec![
        Span::styled("┃", Style::default().fg(COLOR_ACCENT)),
        Span::styled("    ", Style::default()),
        Span::styled("█", Style::default().fg(LOGO_COLORS[3]).bold()),
        Span::styled("█", Style::default().fg(LOGO_COLORS[0]).bold()),
        Span::styled("                 ", Style::default()),
        Span::styled("┃", Style::default().fg(COLOR_PRIMARY)),
        Span::styled("  ╰───────────────────────────────╯", border_style),
    ]);

    // Logo line 5 (G base) + project info
    let line5 = Line::from(vec![
        Span::styled("┗", Style::default().fg(COLOR_ACCENT)),
        Span::styled("  ", Style::default()),
        Span::styled("▀", Style::default().fg(LOGO_COLORS[1]).bold()),
        Span::styled("▀", Style::default().fg(LOGO_COLORS[2]).bold()),
        Span::styled("▀", Style::default().fg(LOGO_COLORS[3]).bold()),
        Span::styled("━━━━━━━━━━━━━━━━━━", border_style),
        Span::styled("┛", Style::default().fg(COLOR_PRIMARY)),
        Span::styled("  ╾╢", border_style),
        Span::styled(" ⬢  ", Style::default().fg(COLOR_SECONDARY)),
        Span::styled(&app.project_id, Style::default().fg(COLOR_SECONDARY).bold()),
        Span::styled(" ╟╼", border_style),
    ]);

    let header = Paragraph::new(vec![line0, line1, line2, line3, line4, line5]);
    frame.render_widget(header, area);
}

// ============================================================================
// Commands Bar - Shows available keyboard shortcuts
// ============================================================================

/// Draws the commands bar showing available actions for current view.
fn draw_commands_bar(frame: &mut Frame, area: Rect, app: &App) {
    let commands = get_commands_for_view(&app.current_view);

    let mut spans: Vec<Span> = vec![Span::styled(" ", Style::default())];

    for (i, (key, desc)) in commands.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(COLOR_BORDER)));
        }
        spans.push(Span::styled(*key, Style::default().fg(COLOR_KEY).bold()));
        spans.push(Span::styled(" ", Style::default()));
        spans.push(Span::styled(*desc, Style::default().fg(COLOR_MUTED)));
    }

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(COLOR_BORDER));

    let commands_widget = Paragraph::new(Line::from(spans))
        .block(block);

    frame.render_widget(commands_widget, area);
}

/// Returns the list of commands available for a given view.
fn get_commands_for_view(view: &View) -> Vec<(&'static str, &'static str)> {
    match view {
        View::AuthRequired => vec![
            ("Enter", "authenticate"),
            ("q", "quit"),
        ],
        View::SecretsList => vec![
            ("j/k", "navigate"),
            ("Enter", "view"),
            ("n", "new secret"),
            ("p", "switch project"),
            ("r", "refresh"),
            ("?", "help"),
            ("q", "quit"),
        ],
        View::ProjectSelector => vec![
            ("j/k", "navigate"),
            ("Enter", "select"),
            ("Esc", "cancel"),
        ],
        View::SecretDetail => vec![
            ("b", "back"),
            ("j/k", "navigate"),
            ("s", "show"),
            ("c", "copy"),
            ("a", "add"),
            ("e/x", "enable/disable"),
            ("p", "project"),
        ],
        View::Input(_) => vec![
            ("Enter", "submit"),
            ("Esc", "cancel"),
        ],
        View::Confirm(_) => vec![
            ("Enter", "confirm"),
            ("Esc", "cancel"),
        ],
    }
}

// ============================================================================
// Status Bar
// ============================================================================

/// Draws the status bar at the bottom (for messages).
fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (text, style) = if let Some(status) = &app.status {
        let style = if status.is_error {
            Style::default().fg(COLOR_ERROR)
        } else {
            Style::default().fg(COLOR_SUCCESS)
        };
        (format!(" {} ", status.text), style)
    } else {
        (" Ready".to_string(), Style::default().fg(COLOR_MUTED))
    };

    let status = Paragraph::new(text).style(style);
    frame.render_widget(status, area);
}

// ============================================================================
// Auth Required View
// ============================================================================

/// Draws the authentication required screen.
fn draw_auth_required(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_WARNING))
        .border_set(symbols::border::ROUNDED)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("", Style::default().fg(COLOR_WARNING)),
            Span::styled(" Authentication Required ", Style::default().fg(COLOR_WARNING).bold()),
        ]));

    let content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("", Style::default().fg(COLOR_WARNING))),
        Line::from(""),
        Line::from(Span::styled(
            "GCP credentials not found",
            Style::default().fg(COLOR_PRIMARY).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "To use this app, you need to authenticate with Google Cloud.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(COLOR_MUTED)),
            Span::styled("Enter", Style::default().fg(COLOR_KEY).bold()),
            Span::styled(" to run: ", Style::default().fg(COLOR_MUTED)),
            Span::styled(
                "gcloud auth application-default login",
                Style::default().fg(COLOR_SECONDARY),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "This will open your browser to authenticate.",
            Style::default().fg(COLOR_MUTED),
        )),
        Line::from(Span::styled(
            "Make sure to check all permission boxes in the consent screen.",
            Style::default().fg(COLOR_WARNING),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(COLOR_MUTED)),
            Span::styled("q", Style::default().fg(COLOR_KEY).bold()),
            Span::styled(" to quit", Style::default().fg(COLOR_MUTED)),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

// ============================================================================
// Secrets List View
// ============================================================================

/// Draws the list of secrets.
fn draw_secrets_list(frame: &mut Frame, area: Rect, app: &App) {
    // Split into header hint and list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Section hint
            Constraint::Min(0),     // List
        ])
        .split(area);

    // Draw section hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("", Style::default().fg(COLOR_WARNING)),
        Span::styled(" ", Style::default()),
        Span::styled("Secrets", Style::default().fg(COLOR_PRIMARY).bold()),
        Span::styled(" - Select a secret to view versions and values", Style::default().fg(COLOR_MUTED)),
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
                Style::default().bg(COLOR_SELECTION).fg(COLOR_SELECTION_TEXT)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled(number, Style::default().fg(COLOR_ACCENT)),
                Span::styled("  ", style),
                Span::styled("", if is_selected { Style::default().fg(COLOR_WARNING) } else { Style::default().fg(COLOR_PRIMARY) }),
                Span::styled(" ", style),
                Span::styled(name, style.add_modifier(Modifier::BOLD)),
                Span::styled("  ", style),
                Span::styled(date, style.fg(if is_selected { COLOR_SELECTION_TEXT } else { COLOR_MUTED })),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    // Create the list widget
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .border_set(symbols::border::ROUNDED)
                .title(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(format!("{}", app.secrets.len()), Style::default().fg(COLOR_SECONDARY).bold()),
                    Span::styled(" secrets ", Style::default().fg(Color::White)),
                ]))
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(Style::default())  // We handle highlighting in items
        .highlight_symbol("");

    frame.render_stateful_widget(list, chunks[1], &mut app.secrets_state.clone());
}

// ============================================================================
// Secret Detail View
// ============================================================================

/// Draws the secret detail view with versions.
fn draw_secret_detail(frame: &mut Frame, area: Rect, app: &App) {
    let secret = match &app.current_secret {
        Some(s) => s,
        None => return,
    };

    // Calculate info card height dynamically based on content
    let mut extra_rows = 0;
    if !secret.labels.is_empty() { extra_rows += 1; }
    if !secret.annotations.is_empty() { extra_rows += 1; }
    if !secret.topics.is_empty() { extra_rows += 1; }
    if !secret.version_aliases.is_empty() { extra_rows += 1; }
    if secret.rotation.is_some() { extra_rows += 1; }
    if secret.version_destroy_ttl.is_some() { extra_rows += 1; }
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
        Span::styled("", Style::default().fg(COLOR_PRIMARY)),
        Span::styled(" ", Style::default()),
        Span::styled("Esc", Style::default().fg(COLOR_KEY).bold()),
        Span::styled(" to go back", Style::default().fg(COLOR_MUTED)),
    ]));
    frame.render_widget(back_hint, chunks[0]);

    // Draw secret info card
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_PRIMARY))
        .border_set(symbols::border::ROUNDED)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("", Style::default().fg(COLOR_PRIMARY)),
            Span::styled(" Secret Details ", Style::default().fg(Color::White).bold()),
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
            Span::styled("  Name        ", Style::default().fg(COLOR_MUTED)),
            Span::styled(&secret.short_name, Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Created     ", Style::default().fg(COLOR_MUTED)),
            Span::styled(&secret.create_time, Style::default().fg(Color::White)),
            Span::styled("    Replication  ", Style::default().fg(COLOR_MUTED)),
            Span::styled(&replication_str, Style::default().fg(COLOR_SECONDARY)),
        ]),
    ];

    // Add labels row if any exist
    if !secret.labels.is_empty() {
        let mut label_spans = vec![
            Span::styled("  Labels      ", Style::default().fg(COLOR_MUTED)),
        ];
        for (i, (key, value)) in secret.labels.iter().enumerate() {
            if i > 0 {
                label_spans.push(Span::styled("  ", Style::default()));
            }
            label_spans.push(Span::styled(key, Style::default().fg(COLOR_ACCENT)));
            label_spans.push(Span::styled("=", Style::default().fg(COLOR_MUTED)));
            label_spans.push(Span::styled(value, Style::default().fg(Color::White)));
        }
        info_content.push(Line::from(label_spans));
    }

    // Add annotations row if any exist
    if !secret.annotations.is_empty() {
        let mut spans = vec![
            Span::styled("  Annotations ", Style::default().fg(COLOR_MUTED)),
        ];
        for (i, (key, value)) in secret.annotations.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ", Style::default()));
            }
            spans.push(Span::styled(key, Style::default().fg(COLOR_WARNING)));
            spans.push(Span::styled("=", Style::default().fg(COLOR_MUTED)));
            spans.push(Span::styled(value, Style::default().fg(Color::White)));
        }
        info_content.push(Line::from(spans));
    }

    // Add topics row if any exist
    if !secret.topics.is_empty() {
        let topics_str = secret.topics.join(", ");
        info_content.push(Line::from(vec![
            Span::styled("  Pub/Sub     ", Style::default().fg(COLOR_MUTED)),
            Span::styled(topics_str, Style::default().fg(Color::White)),
        ]));
    }

    // Add version aliases if any exist
    if !secret.version_aliases.is_empty() {
        let mut spans = vec![
            Span::styled("  Aliases     ", Style::default().fg(COLOR_MUTED)),
        ];
        for (i, (alias, version)) in secret.version_aliases.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ", Style::default()));
            }
            spans.push(Span::styled(alias, Style::default().fg(COLOR_KEY)));
            spans.push(Span::styled("→v", Style::default().fg(COLOR_MUTED)));
            spans.push(Span::styled(version.to_string(), Style::default().fg(Color::White)));
        }
        info_content.push(Line::from(spans));
    }

    // Add rotation config if present
    if let Some(rotation) = &secret.rotation {
        let mut spans = vec![
            Span::styled("  Rotation    ", Style::default().fg(COLOR_MUTED)),
        ];
        if let Some(period) = &rotation.rotation_period {
            spans.push(Span::styled("every ", Style::default().fg(Color::White)));
            spans.push(Span::styled(period, Style::default().fg(COLOR_SECONDARY)));
        }
        if let Some(next) = &rotation.next_rotation_time {
            spans.push(Span::styled("  next: ", Style::default().fg(COLOR_MUTED)));
            spans.push(Span::styled(next, Style::default().fg(Color::White)));
        }
        info_content.push(Line::from(spans));
    }

    // Add version destroy TTL if set
    if let Some(ttl) = &secret.version_destroy_ttl {
        info_content.push(Line::from(vec![
            Span::styled("  Destroy TTL ", Style::default().fg(COLOR_MUTED)),
            Span::styled(ttl, Style::default().fg(COLOR_WARNING)),
            Span::styled(" (delayed destruction)", Style::default().fg(COLOR_MUTED)),
        ]));
    }

    let info = Paragraph::new(info_content).block(info_block);
    frame.render_widget(info, chunks[1]);

    // Draw versions header with action hints
    let versions_hint = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("", Style::default().fg(COLOR_ACCENT)),
        Span::styled(" ", Style::default()),
        Span::styled("Versions", Style::default().fg(COLOR_PRIMARY).bold()),
        Span::styled(" - ", Style::default().fg(COLOR_MUTED)),
        Span::styled("s", Style::default().fg(COLOR_KEY).bold()),
        Span::styled(" show  ", Style::default().fg(COLOR_MUTED)),
        Span::styled("c", Style::default().fg(COLOR_KEY).bold()),
        Span::styled(" copy  ", Style::default().fg(COLOR_MUTED)),
        Span::styled("a", Style::default().fg(COLOR_KEY).bold()),
        Span::styled(" add new", Style::default().fg(COLOR_MUTED)),
    ]));
    frame.render_widget(versions_hint, chunks[2]);

    // Determine if we're showing the secret value
    let (versions_area, value_area) = if app.revealed_value.is_some() {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(7),
            ])
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

/// Draws the versions list.
fn draw_versions_list(frame: &mut Frame, area: Rect, app: &App) {
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
                VersionState::Enabled => ("", COLOR_SUCCESS),
                VersionState::Disabled => ("", COLOR_WARNING),
                VersionState::Destroyed => ("", COLOR_ERROR),
                VersionState::Unknown => ("?", COLOR_MUTED),
            };

            let base_style = if is_selected {
                Style::default().bg(COLOR_SELECTION).fg(COLOR_SELECTION_TEXT)
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
                Span::styled(format!("{:<10}", state_str), base_style.fg(if is_selected { COLOR_SELECTION_TEXT } else { state_color })),
                Span::styled("  ", base_style),
                Span::styled(format!("created {}", create_time), base_style.fg(if is_selected { COLOR_SELECTION_TEXT } else { COLOR_MUTED })),
            ];

            // Add destroy time if destroyed
            if let Some(destroy_time) = &v.destroy_time {
                spans.push(Span::styled("  destroyed ", base_style.fg(if is_selected { COLOR_SELECTION_TEXT } else { COLOR_ERROR })));
                spans.push(Span::styled(destroy_time, base_style.fg(if is_selected { COLOR_SELECTION_TEXT } else { COLOR_MUTED })));
            }

            // Add scheduled destroy time if pending destruction
            if let Some(scheduled) = &v.scheduled_destroy_time {
                spans.push(Span::styled("  ", base_style));
                spans.push(Span::styled("", Style::default().fg(COLOR_WARNING)));
                spans.push(Span::styled(format!(" destroys {}", scheduled), base_style.fg(if is_selected { COLOR_SELECTION_TEXT } else { COLOR_WARNING })));
            }

            // Add checksum indicator
            if v.has_checksum {
                spans.push(Span::styled("  ", base_style));
                spans.push(Span::styled("", Style::default().fg(COLOR_SECONDARY)));
            }

            let content = Line::from(spans);

            ListItem::new(content).style(base_style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .border_set(symbols::border::ROUNDED)
                .title(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(format!("{}", app.versions.len()), Style::default().fg(COLOR_SECONDARY).bold()),
                    Span::styled(" versions ", Style::default().fg(Color::White)),
                ]))
                .padding(Padding::horizontal(1)),
        )
        .highlight_symbol("");

    frame.render_stateful_widget(list, area, &mut app.versions_state.clone());
}

/// Draws the revealed secret value panel.
fn draw_secret_value(frame: &mut Frame, area: Rect, value: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_WARNING))
        .border_set(symbols::border::ROUNDED)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("", Style::default().fg(COLOR_WARNING)),
            Span::styled(" Secret Value ", Style::default().fg(COLOR_WARNING).bold()),
            Span::styled("- press ", Style::default().fg(COLOR_MUTED)),
            Span::styled("s", Style::default().fg(COLOR_KEY).bold()),
            Span::styled(" to hide ", Style::default().fg(COLOR_MUTED)),
        ]))
        .padding(Padding::horizontal(1));

    let content = Paragraph::new(value)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .block(block);

    frame.render_widget(content, area);
}

// ============================================================================
// Empty State
// ============================================================================

/// Draws an empty state with icon, title, and description.
fn draw_empty_state(frame: &mut Frame, area: Rect, title: &str, action: &str, description: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER))
        .border_set(symbols::border::ROUNDED);

    let content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("", Style::default().fg(COLOR_ACCENT))),
        Line::from(""),
        Line::from(Span::styled(title, Style::default().fg(COLOR_PRIMARY).bold())),
        Line::from(""),
        Line::from(Span::styled(action, Style::default().fg(COLOR_SUCCESS))),
        Line::from(""),
        Line::from(Span::styled(description, Style::default().fg(COLOR_MUTED))),
    ];

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

// ============================================================================
// Help Overlay
// ============================================================================

/// Draws a help overlay popup.
fn draw_help_overlay(frame: &mut Frame) {
    let area = centered_rect(65, 75, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let help_text = get_help_text();

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_PRIMARY))
                .border_set(symbols::border::DOUBLE)
                .title(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(" Help ", Style::default().fg(Color::White).bold()),
                    Span::styled("- Press any key to close ", Style::default().fg(COLOR_MUTED)),
                ]))
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help, area);
}

/// Returns the help text content with improved formatting.
fn get_help_text() -> Text<'static> {
    let key_style = Style::default().fg(COLOR_KEY).bold();
    let desc_style = Style::default().fg(Color::White);
    let section_style = Style::default().fg(COLOR_PRIMARY).bold();

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
            Span::styled("or ", Style::default().fg(COLOR_MUTED)),
            Span::styled("Down  ", key_style),
            Span::styled("Move to next item", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("k  ", key_style),
            Span::styled("or ", Style::default().fg(COLOR_MUTED)),
            Span::styled("Up    ", key_style),
            Span::styled("Move to previous item", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("g  ", key_style),
            Span::styled("or ", Style::default().fg(COLOR_MUTED)),
            Span::styled("Home  ", key_style),
            Span::styled("Jump to first item", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("G  ", key_style),
            Span::styled("or ", Style::default().fg(COLOR_MUTED)),
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
            Span::styled("or ", Style::default().fg(COLOR_MUTED)),
            Span::styled("F1    ", key_style),
            Span::styled("Show this help", desc_style),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("q  ", key_style),
            Span::styled("or ", Style::default().fg(COLOR_MUTED)),
            Span::styled("Ctrl+C", key_style),
            Span::styled(" Quit application", desc_style),
        ]),
        Line::from(""),
    ])
}

// ============================================================================
// Input Dialog
// ============================================================================

/// Draws the text input dialog.
fn draw_input_dialog(frame: &mut Frame, mode: &InputMode, app: &App) {
    let (title, prompt, icon) = match mode {
        InputMode::NewSecretName => ("Create New Secret", "Enter a name for your secret:", ""),
        InputMode::NewVersionValue => ("Add New Version", "Enter the secret value:", ""),
    };

    let area = centered_rect(50, 25, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_PRIMARY))
        .border_set(symbols::border::DOUBLE)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(icon, Style::default().fg(COLOR_PRIMARY)),
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
            Span::styled("  ", Style::default()),
            Span::styled(&app.input_buffer, Style::default().fg(Color::White).underlined()),
            Span::styled("", Style::default().fg(COLOR_PRIMARY).add_modifier(Modifier::SLOW_BLINK)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Enter", Style::default().fg(COLOR_KEY).bold()),
            Span::styled(" submit  ", Style::default().fg(COLOR_MUTED)),
            Span::styled("Esc", Style::default().fg(COLOR_KEY).bold()),
            Span::styled(" cancel", Style::default().fg(COLOR_MUTED)),
        ]),
    ];

    let input_widget = Paragraph::new(content).block(block);

    frame.render_widget(input_widget, area);
}

// ============================================================================
// Confirmation Dialog
// ============================================================================

/// Draws the confirmation dialog.
fn draw_confirm_dialog(frame: &mut Frame, action: &ConfirmAction) {
    let (title, message, icon) = match action {
        ConfirmAction::DeleteSecret(name) => (
            "Delete Secret",
            format!(
                "Are you sure you want to delete '{}'?\n\nThis will permanently delete the secret and ALL its versions.\nThis action cannot be undone!",
                name
            ),
            "",
        ),
        ConfirmAction::DestroyVersion(secret, version) => (
            "Destroy Version",
            format!(
                "Are you sure you want to destroy version {} of '{}'?\n\nThe secret data will be permanently destroyed.\nThis action cannot be undone!",
                version, secret
            ),
            "",
        ),
    };

    let area = centered_rect(55, 35, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_ERROR))
        .border_set(symbols::border::DOUBLE)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(icon, Style::default().fg(COLOR_ERROR)),
            Span::styled(" ", Style::default()),
            Span::styled(title, Style::default().fg(COLOR_ERROR).bold()),
            Span::styled(" ", Style::default()),
        ]))
        .padding(Padding::uniform(1));

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(&message, Style::default().fg(COLOR_WARNING))),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Enter", Style::default().fg(COLOR_ERROR).bold()),
            Span::styled(" confirm deletion  ", Style::default().fg(COLOR_MUTED)),
            Span::styled("Esc", Style::default().fg(COLOR_KEY).bold()),
            Span::styled(" cancel", Style::default().fg(COLOR_MUTED)),
        ]),
    ];

    let confirm_widget = Paragraph::new(content)
        .wrap(Wrap { trim: false })
        .block(block);

    frame.render_widget(confirm_widget, area);
}

// ============================================================================
// Project Selector Dialog
// ============================================================================

/// Draws the project selector dialog.
fn draw_project_selector(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    // Split area into title bar, list, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // List
            Constraint::Length(3),  // Footer with commands
        ])
        .margin(1)
        .split(area);

    // Outer block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_PRIMARY))
        .border_set(symbols::border::DOUBLE)
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("", Style::default().fg(COLOR_ACCENT)),
            Span::styled(" Select Project ", Style::default().fg(Color::White).bold()),
        ]));

    frame.render_widget(block, area);

    // Title/hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Current: ", Style::default().fg(COLOR_MUTED)),
        Span::styled(&app.project_id, Style::default().fg(COLOR_SECONDARY).bold()),
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
                Style::default().bg(COLOR_SELECTION).fg(COLOR_SELECTION_TEXT)
            } else {
                Style::default()
            };

            let number = format!("{:>3}", idx + 1);
            let project_id = project.project_id.clone();
            let display_name = if project.display_name != project.project_id {
                format!(" ({})", project.display_name)
            } else {
                String::new()
            };

            let current_marker = if is_current {
                Span::styled(" (current)", Style::default().fg(COLOR_SUCCESS))
            } else {
                Span::raw("")
            };

            let content = Line::from(vec![
                Span::styled(number, Style::default().fg(COLOR_ACCENT)),
                Span::styled("  ", style),
                Span::styled(
                    if is_selected { "" } else { "" },
                    Style::default().fg(if is_current { COLOR_SUCCESS } else { COLOR_PRIMARY }),
                ),
                Span::styled(" ", style),
                Span::styled(project_id, style.add_modifier(Modifier::BOLD)),
                Span::styled(display_name, style.fg(COLOR_MUTED)),
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
        Span::styled("j/k", Style::default().fg(COLOR_KEY).bold()),
        Span::styled(" navigate  ", Style::default().fg(COLOR_MUTED)),
        Span::styled("Enter", Style::default().fg(COLOR_KEY).bold()),
        Span::styled(" select  ", Style::default().fg(COLOR_MUTED)),
        Span::styled("Esc", Style::default().fg(COLOR_KEY).bold()),
        Span::styled(" cancel", Style::default().fg(COLOR_MUTED)),
    ]));
    frame.render_widget(footer, chunks[2]);
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Helper function to create a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

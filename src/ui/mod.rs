//! UI rendering module.
//!
//! This module handles all the terminal UI rendering using Ratatui.
//! Each view is rendered by a separate submodule for clarity.

mod auth;
mod colors;
mod detail;
mod dialogs;
mod empty;
mod header;
mod help;
mod lists;
mod status;
mod utils;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{App, View};
use crate::constants::layout;

// Re-export submodule draw functions for internal use
use auth::draw_auth_required;
use detail::draw_secret_detail;
use dialogs::{draw_confirm_dialog, draw_input_dialog, draw_project_selector};
use header::draw_header;
use help::draw_help_overlay;
use lists::draw_secrets_list;
use status::{draw_commands_bar, draw_status_bar};

/// Main draw function - dispatches to specific view renderers.
pub fn draw(frame: &mut Frame, app: &App) {
    // Create the main layout: header, content, commands bar, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(layout::HEADER_HEIGHT),
            Constraint::Min(0), // Main content
            Constraint::Length(layout::COMMANDS_BAR_HEIGHT),
            Constraint::Length(layout::STATUS_BAR_HEIGHT),
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

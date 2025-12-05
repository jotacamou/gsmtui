//! Google Cloud Secret Manager TUI
//!
//! A terminal user interface for managing Google Cloud secrets.
//! Run with: cargo run -- <PROJECT_ID>

mod app;
mod event;
mod secret_client;
mod ui;

use std::env;

use anyhow::{Context, Result};

use crate::app::{App, View};
use crate::event::EventHandler;

/// Entry point for the application.
///
/// Expects a Google Cloud project ID as the first command-line argument.
/// Make sure you have authenticated with: gcloud auth application-default login
#[tokio::main]
async fn main() -> Result<()> {
    // Get project ID from command line arguments
    let args: Vec<String> = env::args().collect();
    let project_id = args
        .get(1)
        .context("Usage: gsmtui <PROJECT_ID>\n\nPlease provide a Google Cloud project ID.")?
        .clone();

    // Initialize the terminal
    let terminal = ratatui::init();

    // Create the application with the given project ID
    let app = App::new(project_id);

    // Run the application
    let result = run_app(terminal, app).await;

    // Restore the terminal to its original state
    ratatui::restore();

    // Return the result
    result
}

/// Main application loop.
///
/// This function runs the TUI event loop:
/// 1. Draw the current UI state
/// 2. Handle user input events
/// 3. Update application state
/// 4. Repeat until the user quits
async fn run_app(mut terminal: ratatui::DefaultTerminal, mut app: App) -> Result<()> {
    // Load initial data
    app.load_secrets().await?;

    // Create the event handler
    let event_handler = EventHandler::new();

    // Main loop
    loop {
        // Draw the UI
        terminal
            .draw(|frame| ui::draw(frame, &app))
            .context("Failed to draw UI")?;

        // Use different event handling for input mode vs normal mode
        let event = if matches!(app.current_view, View::Input(_)) {
            event_handler.next_input()?
        } else {
            event_handler.next()?
        };

        // Handle events (keyboard input, etc.)
        if let Some(action) = event {
            // Process the event and check if we should quit
            if app.handle_event(action).await? {
                break;
            }
        }
    }

    Ok(())
}

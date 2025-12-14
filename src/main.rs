//! Google Cloud Secret Manager TUI
//!
//! A terminal user interface for managing Google Cloud secrets.
//! Run with: gsmtui [-p|--project <`PROJECT_ID`>]

#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::unused_self
)]

mod app;
mod constants;
mod event;
mod project_client;
mod secret_client;
mod ui;
mod validation;

use std::env;
use std::path::Path;

use anyhow::{Context, Result};

use crate::app::{App, AppAction, View};
use crate::event::EventHandler;

/// Checks if GCP credentials are available.
///
/// Looks for:
/// 1. `GOOGLE_APPLICATION_CREDENTIALS` environment variable pointing to a file
/// 2. Default ADC location: ~/.`config/gcloud/application_default_credentials.json`
fn has_gcp_credentials() -> bool {
    // Check $GOOGLE_APPLICATION_CREDENTIALS first
    if let Ok(path) = env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        return Path::new(&path).exists();
    }

    // Check default ADC location
    if let Ok(home) = env::var("HOME") {
        let adc_path = format!("{home}/.config/gcloud/application_default_credentials.json");
        return Path::new(&adc_path).exists();
    }

    false
}

/// Parses command line arguments.
///
/// Supports:
/// - `-p <PROJECT_ID>` or `--project <PROJECT_ID>` to specify a project
/// - `-h` or `--help` to show usage
///
/// Returns `Some(project_id)` if a project was specified, None otherwise.
fn parse_args() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    // Simple argument parsing using iterator
    let mut args_iter = args.iter().skip(1); // Skip program name

    #[allow(clippy::never_loop)]
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-p" | "--project" => {
                // Get the next argument as the project ID
                if let Some(project_id) = args_iter.next() {
                    return Some(project_id.clone());
                }
                eprintln!("Error: --project requires a PROJECT_ID argument");
                std::process::exit(1);
            }
            "-h" | "--help" => {
                println!("gsmtui - Google Cloud Secret Manager TUI");
                println!();
                println!("Usage: gsmtui [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -p, --project <PROJECT_ID>  Start with the specified GCP project");
                println!("  -h, --help                  Show this help message");
                println!();
                println!("If no project is specified, the project selector will open.");
                println!();
                println!("Make sure you have authenticated with:");
                println!("  gcloud auth application-default login");
                std::process::exit(0);
            }
            other => {
                eprintln!("Error: Unknown argument '{other}'");
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    None
}

/// Entry point for the application.
///
/// If a project ID is provided via -p/--project, loads that project.
/// Otherwise, opens the project selector to choose a project.
///
/// Make sure you have authenticated with: gcloud auth application-default login
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let project_id = parse_args();

    // Initialize the terminal
    let terminal = ratatui::init();

    // Create the application (with optional project ID)
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
    // Check credentials before loading anything
    if has_gcp_credentials() {
        // Load initial data based on starting view
        match app.current_view {
            View::SecretsList => {
                // Project was provided, load secrets
                app.load_secrets().await?;
            }
            View::ProjectSelector => {
                // No project provided, load projects for selection
                app.load_projects().await?;
            }
            _ => {}
        }
    } else {
        app.current_view = View::AuthRequired;
    }

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
            // Process the event and check what action is needed
            match app.handle_event(action).await? {
                Some(AppAction::Quit) => break,
                Some(AppAction::RunGcloudAuth) => {
                    drop(terminal);
                    terminal = run_gcloud_auth(&mut app).await?;
                }
                None => {}
            }
        }
    }

    Ok(())
}

/// Runs gcloud auth with proper terminal management.
///
/// This function:
/// 1. Restores the terminal to normal mode
/// 2. Runs the gcloud auth subprocess
/// 3. Reinitializes the terminal for TUI mode
/// 4. Clears ratatui's buffers to force a full redraw
async fn run_gcloud_auth(app: &mut App) -> Result<ratatui::DefaultTerminal> {
    use std::process::Command;

    // Restore terminal to normal mode
    ratatui::restore();

    // Run gcloud auth - this will open a browser
    let result = Command::new("gcloud")
        .args(["auth", "application-default", "login"])
        .status();

    // Reinitialize terminal for TUI mode
    let mut terminal = ratatui::init();

    // Clear ratatui's internal buffers to force a full redraw
    terminal.clear().context("Failed to clear terminal")?;

    // Handle the result
    match result {
        Ok(status) if status.success() => {
            app.on_auth_success().await?;
        }
        Ok(_) => {
            app.on_auth_failure(None);
        }
        Err(e) => {
            app.on_auth_failure(Some(&e.to_string()));
        }
    }

    Ok(terminal)
}

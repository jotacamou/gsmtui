//! Event handling module.
//!
//! This module handles keyboard and terminal events using crossterm.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::constants::POLL_TIMEOUT;

/// Represents the different actions a user can take in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Quit the application
    Quit,
    /// Move selection up
    Up,
    /// Move selection down
    Down,
    /// Move to the top of the list
    Top,
    /// Move to the bottom of the list
    Bottom,
    /// Select the current item / Enter a submenu
    Enter,
    /// Go back to the previous view
    Back,
    /// Refresh the current view (reload data)
    Refresh,
    /// Create a new secret
    NewSecret,
    /// Add a new version to the current secret
    NewVersion,
    /// Delete the selected item
    Delete,
    /// Copy secret value to clipboard
    Copy,
    /// Toggle showing/hiding secret value
    ToggleSecretValue,
    /// Show help
    Help,
    /// Enable a disabled secret version
    Enable,
    /// Disable an enabled secret version
    Disable,
    /// Open the project selector
    OpenProjectSelector,
    /// Character input (for text entry mode)
    Char(char),
    /// Backspace key (for text entry mode)
    Backspace,
}

/// Handles terminal events and converts them to application actions.
pub struct EventHandler {
    /// Timeout for polling events
    poll_timeout: Duration,
}

impl EventHandler {
    /// Creates a new event handler with default settings.
    pub fn new() -> Self {
        Self {
            poll_timeout: POLL_TIMEOUT,
        }
    }

    /// Polls for the next event and converts it to an Action.
    ///
    /// Returns Ok(None) if no event is available within the timeout.
    /// Returns Ok(Some(action)) if a key event was converted to an action.
    pub fn next(&self) -> io::Result<Option<Action>> {
        // Check if an event is available
        if event::poll(self.poll_timeout)? {
            // Read the event
            if let Event::Key(key_event) = event::read()? {
                // Only process key press events (not releases)
                if key_event.kind == KeyEventKind::Press {
                    return Ok(self.key_to_action(key_event));
                }
            }
        }
        Ok(None)
    }

    /// Polls for input-mode events (for text entry).
    ///
    /// This captures character input and special keys for text editing.
    pub fn next_input(&self) -> io::Result<Option<Action>> {
        if event::poll(self.poll_timeout)? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    return Ok(self.key_to_input_action(key_event));
                }
            }
        }
        Ok(None)
    }

    /// Converts a key event to an input-mode action.
    pub(crate) fn key_to_input_action(&self, key: KeyEvent) -> Option<Action> {
        // Check for Ctrl+C (quit)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Some(Action::Quit);
        }

        match key.code {
            KeyCode::Enter => Some(Action::Enter),
            KeyCode::Esc => Some(Action::Back),
            KeyCode::Backspace => Some(Action::Backspace),
            KeyCode::Char(c) => Some(Action::Char(c)),
            _ => None,
        }
    }

    /// Converts a key event to an application action.
    pub(crate) fn key_to_action(&self, key: KeyEvent) -> Option<Action> {
        // Check for Ctrl+C first (quit)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Some(Action::Quit);
        }

        // Map keys to actions
        match key.code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
            KeyCode::Home | KeyCode::Char('g') => Some(Action::Top),
            KeyCode::End | KeyCode::Char('G') => Some(Action::Bottom),
            KeyCode::Enter => Some(Action::Enter),
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('b') => Some(Action::Back),

            // Actions
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('r') => Some(Action::Refresh),
            KeyCode::Char('n') => Some(Action::NewSecret),
            KeyCode::Char('a') => Some(Action::NewVersion),
            KeyCode::Char('d') => Some(Action::Delete),
            KeyCode::Char('c') => Some(Action::Copy),
            KeyCode::Char('s') => Some(Action::ToggleSecretValue),
            KeyCode::Char('?') | KeyCode::F(1) => Some(Action::Help),
            KeyCode::Char('e') => Some(Action::Enable),
            KeyCode::Char('x') => Some(Action::Disable),
            KeyCode::Char('p') => Some(Action::OpenProjectSelector),

            // No matching action
            _ => None,
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn make_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn make_ctrl_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_vim_navigation_keys() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('j'))),
            Some(Action::Down)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('k'))),
            Some(Action::Up)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('g'))),
            Some(Action::Top)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('G'))),
            Some(Action::Bottom)
        );
    }

    #[test]
    fn test_arrow_navigation_keys() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Up)),
            Some(Action::Up)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Down)),
            Some(Action::Down)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Home)),
            Some(Action::Top)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::End)),
            Some(Action::Bottom)
        );
    }

    #[test]
    fn test_quit_actions() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('q'))),
            Some(Action::Quit)
        );
        assert_eq!(
            handler.key_to_action(make_ctrl_key_event(KeyCode::Char('c'))),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_action_keys() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('n'))),
            Some(Action::NewSecret)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('a'))),
            Some(Action::NewVersion)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('d'))),
            Some(Action::Delete)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('c'))),
            Some(Action::Copy)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('s'))),
            Some(Action::ToggleSecretValue)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('r'))),
            Some(Action::Refresh)
        );
    }

    #[test]
    fn test_help_keys() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('?'))),
            Some(Action::Help)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::F(1))),
            Some(Action::Help)
        );
    }

    #[test]
    fn test_unknown_key_returns_none() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('z'))),
            None
        );
        assert_eq!(handler.key_to_action(make_key_event(KeyCode::F(12))), None);
    }

    #[test]
    fn test_input_mode_actions() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_input_action(make_key_event(KeyCode::Enter)),
            Some(Action::Enter)
        );
        assert_eq!(
            handler.key_to_input_action(make_key_event(KeyCode::Esc)),
            Some(Action::Back)
        );
        assert_eq!(
            handler.key_to_input_action(make_key_event(KeyCode::Backspace)),
            Some(Action::Backspace)
        );
        assert_eq!(
            handler.key_to_input_action(make_key_event(KeyCode::Char('a'))),
            Some(Action::Char('a'))
        );
    }

    #[test]
    fn test_input_mode_ctrl_c_quits() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_input_action(make_ctrl_key_event(KeyCode::Char('c'))),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_enable_disable_keys() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('e'))),
            Some(Action::Enable)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('x'))),
            Some(Action::Disable)
        );
    }

    #[test]
    fn test_project_selector_key() {
        let handler = EventHandler::new();

        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('p'))),
            Some(Action::OpenProjectSelector)
        );
    }

    #[test]
    fn test_back_keys() {
        let handler = EventHandler::new();

        // All three keys should map to Back action
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Esc)),
            Some(Action::Back)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Backspace)),
            Some(Action::Back)
        );
        assert_eq!(
            handler.key_to_action(make_key_event(KeyCode::Char('b'))),
            Some(Action::Back)
        );
    }
}

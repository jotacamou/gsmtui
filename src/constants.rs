//! Application-wide constants.
//!
//! Centralizes magic numbers and configuration values for maintainability.

use std::time::Duration;

/// Event polling timeout - balances responsiveness with CPU usage.
pub const POLL_TIMEOUT: Duration = Duration::from_millis(100);

/// Layout dimensions for the main UI structure.
pub mod layout {
    /// Header height including ASCII art and info panel.
    pub const HEADER_HEIGHT: u16 = 6;
    /// Commands bar height.
    pub const COMMANDS_BAR_HEIGHT: u16 = 3;
    /// Status bar height.
    pub const STATUS_BAR_HEIGHT: u16 = 1;
}

/// Dialog dimensions (percentages of screen size).
pub mod dialog {
    /// Help overlay width percentage.
    pub const HELP_WIDTH: u16 = 65;
    /// Help overlay height percentage.
    pub const HELP_HEIGHT: u16 = 75;
    /// Input dialog width percentage.
    pub const INPUT_WIDTH: u16 = 50;
    /// Input dialog height percentage.
    pub const INPUT_HEIGHT: u16 = 25;
    /// Confirm dialog width percentage.
    pub const CONFIRM_WIDTH: u16 = 55;
    /// Confirm dialog height percentage.
    pub const CONFIRM_HEIGHT: u16 = 35;
    /// Project selector width percentage.
    pub const PROJECT_SELECTOR_WIDTH: u16 = 60;
    /// Project selector height percentage.
    pub const PROJECT_SELECTOR_HEIGHT: u16 = 70;
}

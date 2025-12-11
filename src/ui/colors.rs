//! Color theme definitions for the UI.
//!
//! All color constants are defined here for consistency and easy theme changes.

use ratatui::style::Color;

/// Primary accent color (used for titles, highlights)
pub const PRIMARY: Color = Color::Rgb(56, 189, 248); // Bright cyan
/// Secondary accent color (used for active elements)
pub const SECONDARY: Color = Color::Rgb(52, 211, 153); // Bright emerald
/// Background for selected items
pub const SELECTION: Color = Color::Rgb(99, 102, 241); // Indigo
/// Text on selection
pub const SELECTION_TEXT: Color = Color::White;
/// Muted text color
pub const MUTED: Color = Color::Rgb(148, 163, 184); // Brighter gray
/// Error/danger color
pub const ERROR: Color = Color::Rgb(251, 113, 133); // Bright rose
/// Warning color
pub const WARNING: Color = Color::Rgb(251, 191, 36); // Bright amber
/// Success color
pub const SUCCESS: Color = Color::Rgb(74, 222, 128); // Bright green
/// Border color
pub const BORDER: Color = Color::Rgb(129, 140, 248); // Light indigo
/// Key highlight color (for keyboard shortcuts)
pub const KEY: Color = Color::Rgb(244, 114, 182); // Bright pink
/// Accent color for icons and decorations
pub const ACCENT: Color = Color::Rgb(192, 132, 252); // Bright purple

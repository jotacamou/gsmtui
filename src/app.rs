//! Application state and logic.
//!
//! This module contains the core application state, view management,
//! and event handling logic.

use anyhow::Result;
use ratatui::widgets::ListState;

use crate::event::Action;
use crate::secret_client::{SecretClient, SecretInfo, VersionInfo};

/// The different views/screens in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// List of all secrets in the project
    SecretsList,
    /// Details and versions of a specific secret
    SecretDetail,
    /// Help screen showing keyboard shortcuts
    Help,
    /// Text input mode (for creating secrets, adding values, etc.)
    Input(InputMode),
    /// Confirmation dialog (for destructive actions)
    Confirm(ConfirmAction),
}

/// Different input modes for text entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    /// Creating a new secret (entering the name)
    NewSecretName,
    /// Adding a new version (entering the value)
    NewVersionValue,
}

/// Actions that require confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Delete a secret
    DeleteSecret(String),
    /// Destroy a secret version
    DestroyVersion(String, String),
}

/// Status message to display to the user.
#[derive(Debug, Clone)]
pub struct StatusMessage {
    /// The message text
    pub text: String,
    /// Whether this is an error message
    pub is_error: bool,
}

/// Main application state.
pub struct App {
    /// Google Cloud project ID
    pub project_id: String,
    /// Secret Manager client (initialized lazily)
    client: Option<SecretClient>,
    /// Current view/screen
    pub current_view: View,
    /// Previous view (for going back)
    pub previous_view: Option<View>,
    /// Is the app loading data?
    pub is_loading: bool,
    /// Status message to display
    pub status: Option<StatusMessage>,

    // --- Secrets list state ---
    /// List of secrets
    pub secrets: Vec<SecretInfo>,
    /// Selection state for the secrets list
    pub secrets_state: ListState,

    // --- Secret detail state ---
    /// Currently selected secret (when viewing details)
    pub current_secret: Option<SecretInfo>,
    /// Versions of the current secret
    pub versions: Vec<VersionInfo>,
    /// Selection state for the versions list
    pub versions_state: ListState,
    /// Currently visible secret value (if revealed)
    pub revealed_value: Option<String>,

    // --- Input state ---
    /// Current input buffer for text entry
    pub input_buffer: String,

    // --- Help visibility ---
    pub show_help: bool,
}

impl App {
    /// Creates a new application instance for the given project.
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            client: None,
            current_view: View::SecretsList,
            previous_view: None,
            is_loading: false,
            status: None,
            secrets: Vec::new(),
            secrets_state: ListState::default(),
            current_secret: None,
            versions: Vec::new(),
            versions_state: ListState::default(),
            revealed_value: None,
            input_buffer: String::new(),
            show_help: false,
        }
    }

    /// Returns a reference to the Secret Manager client.
    /// Initializes it if not already done.
    async fn get_client(&mut self) -> Result<&SecretClient> {
        if self.client.is_none() {
            self.client = Some(SecretClient::new(self.project_id.clone()).await?);
        }
        Ok(self.client.as_ref().unwrap())
    }

    /// Loads the list of secrets from the API.
    pub async fn load_secrets(&mut self) -> Result<()> {
        self.is_loading = true;
        self.set_status("Loading secrets...", false);

        // We need to work around the borrow checker here
        let project_id = self.project_id.clone();
        if self.client.is_none() {
            self.client = Some(SecretClient::new(project_id).await?);
        }

        match self.client.as_ref().unwrap().list_secrets().await {
            Ok(secrets) => {
                self.secrets = secrets;
                // Select the first item if list is not empty
                if !self.secrets.is_empty() {
                    self.secrets_state.select(Some(0));
                }
                let count = self.secrets.len();
                self.set_status(&format!("Loaded {} secrets", count), false);
            }
            Err(e) => {
                self.set_status(&format!("Error loading secrets: {}", e), true);
            }
        }

        self.is_loading = false;
        Ok(())
    }

    /// Loads versions for the currently selected secret.
    pub async fn load_versions(&mut self) -> Result<()> {
        let secret_name = match &self.current_secret {
            Some(s) => s.short_name.clone(),
            None => return Ok(()),
        };

        self.is_loading = true;
        self.set_status("Loading versions...", false);

        match self.client.as_ref().unwrap().list_versions(&secret_name).await {
            Ok(versions) => {
                self.versions = versions;
                // Select the first version if list is not empty
                if !self.versions.is_empty() {
                    self.versions_state.select(Some(0));
                }
                let count = self.versions.len();
                self.set_status(&format!("Loaded {} versions", count), false);
            }
            Err(e) => {
                self.set_status(&format!("Error loading versions: {}", e), true);
            }
        }

        self.is_loading = false;
        Ok(())
    }

    /// Handles an action and returns true if the app should quit.
    pub async fn handle_event(&mut self, action: Action) -> Result<bool> {
        // Handle help toggle from any view
        if action == Action::Help {
            self.show_help = !self.show_help;
            return Ok(false);
        }

        // If help is showing, any key closes it
        if self.show_help {
            self.show_help = false;
            return Ok(false);
        }

        // Handle confirmation dialogs
        if let View::Confirm(ref confirm_action) = self.current_view {
            return self.handle_confirm_action(action, confirm_action.clone()).await;
        }

        // Handle input mode
        if let View::Input(ref input_mode) = self.current_view {
            return self.handle_input_action(action, input_mode.clone()).await;
        }

        // Handle based on current view
        match self.current_view {
            View::SecretsList => self.handle_secrets_list_action(action).await,
            View::SecretDetail => self.handle_secret_detail_action(action).await,
            View::Help => {
                // Any key exits help
                self.go_back();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Handles actions in the secrets list view.
    async fn handle_secrets_list_action(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Quit => return Ok(true),
            Action::Up => self.select_previous_secret(),
            Action::Down => self.select_next_secret(),
            Action::Top => self.select_first_secret(),
            Action::Bottom => self.select_last_secret(),
            Action::Enter => self.enter_secret_detail().await?,
            Action::Refresh => self.load_secrets().await?,
            Action::NewSecret => self.start_new_secret(),
            Action::Delete => self.confirm_delete_secret(),
            _ => {}
        }
        Ok(false)
    }

    /// Handles actions in the secret detail view.
    async fn handle_secret_detail_action(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Quit => return Ok(true),
            Action::Back => self.go_back(),
            Action::Up => self.select_previous_version(),
            Action::Down => self.select_next_version(),
            Action::Top => self.select_first_version(),
            Action::Bottom => self.select_last_version(),
            Action::Refresh => self.load_versions().await?,
            Action::NewVersion => self.start_new_version(),
            Action::ToggleSecretValue => self.toggle_secret_value().await?,
            Action::Copy => self.copy_secret_value().await?,
            Action::Enable => self.enable_selected_version().await?,
            Action::Disable => self.disable_selected_version().await?,
            Action::Delete => self.confirm_destroy_version(),
            _ => {}
        }
        Ok(false)
    }

    /// Handles actions during text input.
    async fn handle_input_action(&mut self, action: Action, mode: InputMode) -> Result<bool> {
        match action {
            Action::Quit => return Ok(true),
            Action::Back => {
                self.input_buffer.clear();
                self.go_back();
            }
            Action::Enter => {
                self.submit_input(mode).await?;
            }
            Action::Char(c) => {
                self.input_char(c);
            }
            Action::Backspace => {
                self.input_backspace();
            }
            _ => {}
        }
        Ok(false)
    }

    /// Handles actions in confirmation dialogs.
    async fn handle_confirm_action(&mut self, action: Action, confirm: ConfirmAction) -> Result<bool> {
        match action {
            Action::Enter => {
                // User confirmed the action
                self.execute_confirmed_action(confirm).await?;
            }
            Action::Back | Action::Quit => {
                // User cancelled
                self.go_back();
            }
            _ => {}
        }
        Ok(false)
    }

    // --- Navigation helpers ---

    fn select_previous_secret(&mut self) {
        let len = self.secrets.len();
        if len == 0 {
            return;
        }
        let current = self.secrets_state.selected().unwrap_or(0);
        let new = if current == 0 { len - 1 } else { current - 1 };
        self.secrets_state.select(Some(new));
    }

    fn select_next_secret(&mut self) {
        let len = self.secrets.len();
        if len == 0 {
            return;
        }
        let current = self.secrets_state.selected().unwrap_or(0);
        let new = if current >= len - 1 { 0 } else { current + 1 };
        self.secrets_state.select(Some(new));
    }

    fn select_first_secret(&mut self) {
        if !self.secrets.is_empty() {
            self.secrets_state.select(Some(0));
        }
    }

    fn select_last_secret(&mut self) {
        let len = self.secrets.len();
        if len > 0 {
            self.secrets_state.select(Some(len - 1));
        }
    }

    fn select_previous_version(&mut self) {
        let len = self.versions.len();
        if len == 0 {
            return;
        }
        let current = self.versions_state.selected().unwrap_or(0);
        let new = if current == 0 { len - 1 } else { current - 1 };
        self.versions_state.select(Some(new));
        self.revealed_value = None; // Hide value when selection changes
    }

    fn select_next_version(&mut self) {
        let len = self.versions.len();
        if len == 0 {
            return;
        }
        let current = self.versions_state.selected().unwrap_or(0);
        let new = if current >= len - 1 { 0 } else { current + 1 };
        self.versions_state.select(Some(new));
        self.revealed_value = None; // Hide value when selection changes
    }

    fn select_first_version(&mut self) {
        if !self.versions.is_empty() {
            self.versions_state.select(Some(0));
            self.revealed_value = None;
        }
    }

    fn select_last_version(&mut self) {
        let len = self.versions.len();
        if len > 0 {
            self.versions_state.select(Some(len - 1));
            self.revealed_value = None;
        }
    }

    /// Enters the detail view for the selected secret.
    async fn enter_secret_detail(&mut self) -> Result<()> {
        if let Some(idx) = self.secrets_state.selected() {
            if let Some(secret) = self.secrets.get(idx) {
                self.current_secret = Some(secret.clone());
                self.previous_view = Some(View::SecretsList);
                self.current_view = View::SecretDetail;
                self.versions_state = ListState::default();
                self.revealed_value = None;
                self.load_versions().await?;
            }
        }
        Ok(())
    }

    /// Goes back to the previous view.
    fn go_back(&mut self) {
        if let Some(prev) = self.previous_view.take() {
            self.current_view = prev;
        } else {
            self.current_view = View::SecretsList;
        }
        self.revealed_value = None;
    }

    // --- Input handling ---

    fn start_new_secret(&mut self) {
        self.input_buffer.clear();
        self.previous_view = Some(self.current_view.clone());
        self.current_view = View::Input(InputMode::NewSecretName);
    }

    fn start_new_version(&mut self) {
        self.input_buffer.clear();
        self.previous_view = Some(self.current_view.clone());
        self.current_view = View::Input(InputMode::NewVersionValue);
    }

    async fn submit_input(&mut self, mode: InputMode) -> Result<()> {
        let input = self.input_buffer.clone();
        self.input_buffer.clear();

        if input.is_empty() {
            self.set_status("Input cannot be empty", true);
            self.go_back();
            return Ok(());
        }

        match mode {
            InputMode::NewSecretName => {
                self.is_loading = true;
                match self.client.as_ref().unwrap().create_secret(&input).await {
                    Ok(_) => {
                        self.set_status(&format!("Created secret: {}", input), false);
                        self.go_back();
                        self.load_secrets().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to create secret: {}", e), true);
                        self.go_back();
                    }
                }
                self.is_loading = false;
            }
            InputMode::NewVersionValue => {
                if let Some(secret) = &self.current_secret {
                    let secret_name = secret.short_name.clone();
                    self.is_loading = true;
                    match self.client.as_ref().unwrap().add_version(&secret_name, &input).await {
                        Ok(v) => {
                            self.set_status(&format!("Added version: {}", v.version), false);
                            self.go_back();
                            self.load_versions().await?;
                        }
                        Err(e) => {
                            self.set_status(&format!("Failed to add version: {}", e), true);
                            self.go_back();
                        }
                    }
                    self.is_loading = false;
                }
            }
        }
        Ok(())
    }

    /// Appends a character to the input buffer.
    pub fn input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    /// Removes the last character from the input buffer.
    pub fn input_backspace(&mut self) {
        self.input_buffer.pop();
    }

    // --- Confirmation dialogs ---

    fn confirm_delete_secret(&mut self) {
        if let Some(idx) = self.secrets_state.selected() {
            if let Some(secret) = self.secrets.get(idx) {
                let secret_name = secret.short_name.clone();
                self.previous_view = Some(self.current_view.clone());
                self.current_view = View::Confirm(ConfirmAction::DeleteSecret(secret_name));
            }
        }
    }

    fn confirm_destroy_version(&mut self) {
        if let (Some(secret), Some(idx)) = (&self.current_secret, self.versions_state.selected()) {
            if let Some(version) = self.versions.get(idx) {
                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();
                self.previous_view = Some(self.current_view.clone());
                self.current_view = View::Confirm(ConfirmAction::DestroyVersion(secret_name, version_num));
            }
        }
    }

    async fn execute_confirmed_action(&mut self, action: ConfirmAction) -> Result<()> {
        match action {
            ConfirmAction::DeleteSecret(name) => {
                self.is_loading = true;
                match self.client.as_ref().unwrap().delete_secret(&name).await {
                    Ok(()) => {
                        self.set_status(&format!("Deleted secret: {}", name), false);
                        self.current_view = View::SecretsList;
                        self.previous_view = None;
                        self.load_secrets().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to delete: {}", e), true);
                        self.go_back();
                    }
                }
                self.is_loading = false;
            }
            ConfirmAction::DestroyVersion(secret_name, version) => {
                self.is_loading = true;
                match self.client.as_ref().unwrap().destroy_version(&secret_name, &version).await {
                    Ok(_) => {
                        self.set_status(&format!("Destroyed version: {}", version), false);
                        self.go_back();
                        self.load_versions().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to destroy: {}", e), true);
                        self.go_back();
                    }
                }
                self.is_loading = false;
            }
        }
        Ok(())
    }

    // --- Secret value operations ---

    async fn toggle_secret_value(&mut self) -> Result<()> {
        // If already showing, hide it
        if self.revealed_value.is_some() {
            self.revealed_value = None;
            return Ok(());
        }

        // Otherwise, fetch and show the value
        if let (Some(secret), Some(idx)) = (&self.current_secret, self.versions_state.selected()) {
            if let Some(version) = self.versions.get(idx) {
                // Only show for enabled versions (API restriction)
                if is_version_destroyed(&version.state) {
                    self.set_status("Cannot access destroyed version - data is permanently gone", true);
                    return Ok(());
                }
                if is_version_disabled(&version.state) {
                    self.set_status("Version is disabled - press 'e' to enable it first", true);
                    return Ok(());
                }

                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();

                self.is_loading = true;
                match self.client.as_ref().unwrap().access_version(&secret_name, &version_num).await {
                    Ok(value) => {
                        self.revealed_value = Some(value);
                        self.set_status("Press 's' to hide value", false);
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to access: {}", e), true);
                    }
                }
                self.is_loading = false;
            }
        }
        Ok(())
    }

    async fn copy_secret_value(&mut self) -> Result<()> {
        if let (Some(secret), Some(idx)) = (&self.current_secret, self.versions_state.selected()) {
            if let Some(version) = self.versions.get(idx) {
                if is_version_destroyed(&version.state) {
                    self.set_status("Cannot copy destroyed version - data is permanently gone", true);
                    return Ok(());
                }
                if is_version_disabled(&version.state) {
                    self.set_status("Version is disabled - press 'e' to enable it first", true);
                    return Ok(());
                }

                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();

                self.is_loading = true;
                match self.client.as_ref().unwrap().access_version(&secret_name, &version_num).await {
                    Ok(value) => {
                        // Try to copy to clipboard
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => {
                                if clipboard.set_text(&value).is_ok() {
                                    self.set_status("Copied to clipboard!", false);
                                } else {
                                    self.set_status("Failed to copy to clipboard", true);
                                }
                            }
                            Err(_) => {
                                self.set_status("Clipboard not available", true);
                            }
                        }
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to access: {}", e), true);
                    }
                }
                self.is_loading = false;
            }
        }
        Ok(())
    }

    // --- Version state operations ---

    async fn enable_selected_version(&mut self) -> Result<()> {
        if let (Some(secret), Some(idx)) = (&self.current_secret, self.versions_state.selected()) {
            if let Some(version) = self.versions.get(idx) {
                if !is_version_disabled(&version.state) {
                    self.set_status("Can only enable disabled versions", true);
                    return Ok(());
                }

                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();

                self.is_loading = true;
                match self.client.as_ref().unwrap().enable_version(&secret_name, &version_num).await {
                    Ok(_) => {
                        self.set_status(&format!("Enabled version: {}", version_num), false);
                        self.load_versions().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to enable: {}", e), true);
                    }
                }
                self.is_loading = false;
            }
        }
        Ok(())
    }

    async fn disable_selected_version(&mut self) -> Result<()> {
        if let (Some(secret), Some(idx)) = (&self.current_secret, self.versions_state.selected()) {
            if let Some(version) = self.versions.get(idx) {
                if !is_version_enabled(&version.state) {
                    self.set_status("Can only disable enabled versions", true);
                    return Ok(());
                }

                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();

                self.is_loading = true;
                match self.client.as_ref().unwrap().disable_version(&secret_name, &version_num).await {
                    Ok(_) => {
                        self.set_status(&format!("Disabled version: {}", version_num), false);
                        self.load_versions().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to disable: {}", e), true);
                    }
                }
                self.is_loading = false;
            }
        }
        Ok(())
    }

    // --- Status message helpers ---

    fn set_status(&mut self, text: &str, is_error: bool) {
        self.status = Some(StatusMessage {
            text: text.to_string(),
            is_error,
        });
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    /// Returns the currently selected secret (if any).
    pub fn selected_secret(&self) -> Option<&SecretInfo> {
        self.secrets_state
            .selected()
            .and_then(|idx| self.secrets.get(idx))
    }

    /// Returns the currently selected version (if any).
    pub fn selected_version(&self) -> Option<&VersionInfo> {
        self.versions_state
            .selected()
            .and_then(|idx| self.versions.get(idx))
    }
}

// ============================================================================
// Helper Functions for Version State Checks
// ============================================================================

/// Checks if a version state string indicates the version is enabled.
/// The state comes from the API as a debug-formatted enum (e.g., "Enabled", "State::Enabled").
fn is_version_enabled(state: &str) -> bool {
    let state_lower = state.to_lowercase();
    state_lower.contains("enabled") && !state_lower.contains("disabled")
}

/// Checks if a version state string indicates the version is disabled.
fn is_version_disabled(state: &str) -> bool {
    state.to_lowercase().contains("disabled")
}

/// Checks if a version state string indicates the version is destroyed.
#[allow(dead_code)]
fn is_version_destroyed(state: &str) -> bool {
    state.to_lowercase().contains("destroyed")
}

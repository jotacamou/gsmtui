//! Application state and logic.
//!
//! This module contains the core application state, view management,
//! and event handling logic.

use anyhow::Result;
use ratatui::widgets::ListState;

use crate::event::Action;
use crate::project_client::{self, ProjectInfo};
use crate::secret_client::{SecretClient, SecretInfo, VersionInfo, VersionState};

/// The different views/screens in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// Authentication required - credentials not found
    AuthRequired,
    /// List of all secrets in the project
    SecretsList,
    /// Details and versions of a specific secret
    SecretDetail,
    /// Text input mode (for creating secrets, adding values, etc.)
    Input(InputMode),
    /// Confirmation dialog (for destructive actions)
    Confirm(ConfirmAction),
    /// Project selector dialog
    ProjectSelector,
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

/// Actions that need to be handled by the main loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    /// Quit the application
    Quit,
    /// Run gcloud auth (needs terminal access)
    RunGcloudAuth,
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
    /// Cursor position within the input buffer (character index)
    pub cursor_position: usize,

    // --- Help visibility ---
    pub show_help: bool,

    // --- Project selector state ---
    /// List of available GCP projects
    pub available_projects: Vec<ProjectInfo>,
    /// Selection state for the projects list
    pub projects_state: ListState,
}

impl App {
    /// Creates a new application instance.
    ///
    /// If a `project_id` is provided, starts in `SecretsList` view.
    /// If None, starts in `ProjectSelector` view for the user to choose a project.
    pub fn new(project_id: Option<String>) -> Self {
        // Determine initial view and project based on whether a project was provided
        let (initial_view, project) = match project_id {
            Some(id) => (View::SecretsList, id),
            None => (View::ProjectSelector, String::new()),
        };

        Self {
            project_id: project,
            client: None,
            current_view: initial_view,
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
            cursor_position: 0,
            show_help: false,
            available_projects: Vec::new(),
            projects_state: ListState::default(),
        }
    }

    /// Loads the list of secrets from the API.
    /// If loading fails (likely auth issue), switches to `AuthRequired` view.
    pub async fn load_secrets(&mut self) -> Result<()> {
        self.is_loading = true;
        self.set_status("Loading secrets...", false);

        // Initialize client if needed
        let project_id = self.project_id.clone();
        if self.client.is_none() {
            match SecretClient::new(project_id).await {
                Ok(c) => self.client = Some(c),
                Err(e) => {
                    self.set_status(&format!("Auth error: {e}"), true);
                    self.current_view = View::AuthRequired;
                    self.is_loading = false;
                    return Ok(());
                }
            }
        }

        match self.client.as_ref().unwrap().list_secrets().await {
            Ok(secrets) => {
                self.secrets = secrets;
                // Select the first item if list is not empty
                if !self.secrets.is_empty() {
                    self.secrets_state.select(Some(0));
                }
                let count = self.secrets.len();
                self.set_status(&format!("Loaded {count} secrets"), false);
            }
            Err(e) => {
                self.set_status(&format!("Auth error: {e}"), true);
                self.current_view = View::AuthRequired;
            }
        }

        self.is_loading = false;
        Ok(())
    }

    /// Loads the list of available projects from the API.
    ///
    /// Used when starting without a project or when opening the project selector.
    /// If loading fails (likely auth issue), switches to `AuthRequired` view.
    pub async fn load_projects(&mut self) -> Result<()> {
        self.is_loading = true;
        self.set_status("Loading projects...", false);

        match project_client::list_projects().await {
            Ok(projects) => {
                self.available_projects = projects;
                // Try to select the current project in the list, or first item
                let current_idx = if self.project_id.is_empty() {
                    0
                } else {
                    self.available_projects
                        .iter()
                        .position(|p| p.project_id == self.project_id)
                        .unwrap_or(0)
                };
                if !self.available_projects.is_empty() {
                    self.projects_state.select(Some(current_idx));
                }
                let count = self.available_projects.len();
                self.set_status(&format!("Found {count} projects"), false);
            }
            Err(e) => {
                // Loading failed - likely an auth issue, switch to auth view
                self.set_status(&format!("Auth error: {e}"), true);
                self.current_view = View::AuthRequired;
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

        match self
            .client
            .as_ref()
            .unwrap()
            .list_versions(&secret_name)
            .await
        {
            Ok(versions) => {
                self.versions = versions;
                // Select the first version if list is not empty
                if !self.versions.is_empty() {
                    self.versions_state.select(Some(0));
                }
                let count = self.versions.len();
                self.set_status(&format!("Loaded {count} versions"), false);
            }
            Err(e) => {
                self.set_status(&format!("Error loading versions: {e}"), true);
            }
        }

        self.is_loading = false;
        Ok(())
    }

    /// Handles an action and returns an `AppAction` if one is needed.
    pub async fn handle_event(&mut self, action: Action) -> Result<Option<AppAction>> {
        // Handle help toggle from any view
        if action == Action::Help {
            self.show_help = !self.show_help;
            return Ok(None);
        }

        // If help is showing, any key closes it
        if self.show_help {
            self.show_help = false;
            return Ok(None);
        }

        // Handle confirmation dialogs
        if let View::Confirm(ref confirm_action) = self.current_view {
            return self
                .handle_confirm_action(action, confirm_action.clone())
                .await;
        }

        // Handle input mode
        if let View::Input(ref input_mode) = self.current_view {
            return self.handle_input_action(action, input_mode.clone()).await;
        }

        // Handle based on current view
        match self.current_view {
            View::AuthRequired => Ok(self.handle_auth_required_action(&action)),
            View::SecretsList => self.handle_secrets_list_action(action).await,
            View::SecretDetail => self.handle_secret_detail_action(action).await,
            View::ProjectSelector => self.handle_project_selector_action(action).await,
            _ => Ok(None),
        }
    }

    /// Handles actions in the auth required view.
    fn handle_auth_required_action(&mut self, action: &Action) -> Option<AppAction> {
        match action {
            Action::Quit => Some(AppAction::Quit),
            Action::Enter => Some(AppAction::RunGcloudAuth),
            _ => None,
        }
    }

    /// Called after successful gcloud auth to load projects.
    pub async fn on_auth_success(&mut self) -> Result<()> {
        self.set_status("Authentication successful!", false);
        self.current_view = View::ProjectSelector;
        self.load_projects().await
    }

    /// Called when gcloud auth fails.
    pub fn on_auth_failure(&mut self, error: Option<&str>) {
        if let Some(e) = error {
            self.set_status(&format!("Failed to run gcloud: {e}"), true);
        } else {
            self.set_status("Authentication was cancelled or failed", true);
        }
    }

    /// Handles actions in the secrets list view.
    async fn handle_secrets_list_action(&mut self, action: Action) -> Result<Option<AppAction>> {
        match action {
            Action::Quit => return Ok(Some(AppAction::Quit)),
            Action::Up => self.select_previous_secret(),
            Action::Down => self.select_next_secret(),
            Action::Top => self.select_first_secret(),
            Action::Bottom => self.select_last_secret(),
            Action::Enter => self.enter_secret_detail().await?,
            Action::Refresh => self.load_secrets().await?,
            Action::NewSecret => self.start_new_secret(),
            Action::Delete => self.confirm_delete_secret(),
            Action::OpenProjectSelector => self.open_project_selector().await?,
            _ => {}
        }
        Ok(None)
    }

    /// Handles actions in the project selector view.
    async fn handle_project_selector_action(
        &mut self,
        action: Action,
    ) -> Result<Option<AppAction>> {
        match action {
            Action::Quit => return Ok(Some(AppAction::Quit)),
            Action::Back => self.go_back(),
            Action::Up => self.select_previous_project(),
            Action::Down => self.select_next_project(),
            Action::Top => self.select_first_project(),
            Action::Bottom => self.select_last_project(),
            Action::Enter => self.select_project().await?,
            _ => {}
        }
        Ok(None)
    }

    /// Handles actions in the secret detail view.
    async fn handle_secret_detail_action(&mut self, action: Action) -> Result<Option<AppAction>> {
        match action {
            Action::Quit => return Ok(Some(AppAction::Quit)),
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
            Action::OpenProjectSelector => self.open_project_selector().await?,
            _ => {}
        }
        Ok(None)
    }

    /// Handles actions during text input.
    async fn handle_input_action(
        &mut self,
        action: Action,
        mode: InputMode,
    ) -> Result<Option<AppAction>> {
        match action {
            Action::Quit => return Ok(Some(AppAction::Quit)),
            Action::Back => {
                self.input_buffer.clear();
                self.cursor_position = 0;
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
            Action::CursorLeft => {
                self.cursor_left();
            }
            Action::CursorRight => {
                self.cursor_right();
            }
            _ => {}
        }
        Ok(None)
    }

    /// Handles actions in confirmation dialogs.
    async fn handle_confirm_action(
        &mut self,
        action: Action,
        confirm: ConfirmAction,
    ) -> Result<Option<AppAction>> {
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
        Ok(None)
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

    // --- Project navigation helpers ---

    fn select_previous_project(&mut self) {
        let len = self.available_projects.len();
        if len == 0 {
            return;
        }
        let current = self.projects_state.selected().unwrap_or(0);
        let new = if current == 0 { len - 1 } else { current - 1 };
        self.projects_state.select(Some(new));
    }

    fn select_next_project(&mut self) {
        let len = self.available_projects.len();
        if len == 0 {
            return;
        }
        let current = self.projects_state.selected().unwrap_or(0);
        let new = if current >= len - 1 { 0 } else { current + 1 };
        self.projects_state.select(Some(new));
    }

    fn select_first_project(&mut self) {
        if !self.available_projects.is_empty() {
            self.projects_state.select(Some(0));
        }
    }

    fn select_last_project(&mut self) {
        let len = self.available_projects.len();
        if len > 0 {
            self.projects_state.select(Some(len - 1));
        }
    }

    /// Opens the project selector dialog.
    async fn open_project_selector(&mut self) -> Result<()> {
        // Load the projects
        self.load_projects().await?;

        // Switch to project selector view
        self.previous_view = Some(self.current_view.clone());
        self.current_view = View::ProjectSelector;

        Ok(())
    }

    /// Selects a project and switches to it.
    async fn select_project(&mut self) -> Result<()> {
        if let Some(idx) = self.projects_state.selected() {
            if let Some(project) = self.available_projects.get(idx) {
                let new_project_id = project.project_id.clone();

                // Don't reload if same project
                if new_project_id == self.project_id {
                    self.set_status("Already on this project", false);
                    self.go_back();
                    return Ok(());
                }

                // Switch to the new project
                self.project_id.clone_from(&new_project_id);
                self.client = None; // Clear the client to force reinitialization
                self.secrets.clear();
                self.secrets_state = ListState::default();
                self.current_secret = None;
                self.versions.clear();
                self.versions_state = ListState::default();
                self.revealed_value = None;

                self.set_status(&format!("Switched to project: {new_project_id}"), false);
                self.current_view = View::SecretsList;
                self.previous_view = None;

                // Load secrets for the new project
                self.load_secrets().await?;
            }
        }
        Ok(())
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
        self.cursor_position = 0;
        self.previous_view = Some(self.current_view.clone());
        self.current_view = View::Input(InputMode::NewSecretName);
    }

    fn start_new_version(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.previous_view = Some(self.current_view.clone());
        self.current_view = View::Input(InputMode::NewVersionValue);
    }

    async fn submit_input(&mut self, mode: InputMode) -> Result<()> {
        let input = self.input_buffer.clone();
        self.input_buffer.clear();
        self.cursor_position = 0;

        if input.is_empty() {
            self.set_status("Input cannot be empty", true);
            self.go_back();
            return Ok(());
        }

        match mode {
            InputMode::NewSecretName => {
                // Validate secret name before API call
                if let Err(e) = crate::validation::validate_secret_name(&input) {
                    self.set_status(&e, true);
                    self.go_back();
                    return Ok(());
                }

                self.is_loading = true;
                match self.client.as_ref().unwrap().create_secret(&input).await {
                    Ok(_) => {
                        self.set_status(&format!("Created secret: {input}"), false);
                        self.go_back();
                        self.load_secrets().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to create secret: {e}"), true);
                        self.go_back();
                    }
                }
                self.is_loading = false;
            }
            InputMode::NewVersionValue => {
                if let Some(secret) = &self.current_secret {
                    let secret_name = secret.short_name.clone();
                    self.is_loading = true;
                    match self
                        .client
                        .as_ref()
                        .unwrap()
                        .add_version(&secret_name, &input)
                        .await
                    {
                        Ok(v) => {
                            self.set_status(&format!("Added version: {}", v.version), false);
                            self.go_back();
                            self.load_versions().await?;
                        }
                        Err(e) => {
                            self.set_status(&format!("Failed to add version: {e}"), true);
                            self.go_back();
                        }
                    }
                    self.is_loading = false;
                }
            }
        }
        Ok(())
    }

    /// Inserts a character at the current cursor position.
    pub fn input_char(&mut self, c: char) {
        // Convert to char indices for proper Unicode handling
        let byte_idx = self
            .input_buffer
            .char_indices()
            .nth(self.cursor_position)
            .map(|(i, _)| i)
            .unwrap_or(self.input_buffer.len());
        self.input_buffer.insert(byte_idx, c);
        self.cursor_position += 1;
    }

    /// Removes the character before the cursor position.
    pub fn input_backspace(&mut self) {
        if self.cursor_position > 0 {
            // Find the byte index of the character to remove
            let byte_idx = self
                .input_buffer
                .char_indices()
                .nth(self.cursor_position - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input_buffer.remove(byte_idx);
            self.cursor_position -= 1;
        }
    }

    /// Moves the cursor one position to the left.
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Moves the cursor one position to the right.
    pub fn cursor_right(&mut self) {
        let char_count = self.input_buffer.chars().count();
        if self.cursor_position < char_count {
            self.cursor_position += 1;
        }
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
                self.current_view =
                    View::Confirm(ConfirmAction::DestroyVersion(secret_name, version_num));
            }
        }
    }

    async fn execute_confirmed_action(&mut self, action: ConfirmAction) -> Result<()> {
        match action {
            ConfirmAction::DeleteSecret(name) => {
                self.is_loading = true;
                match self.client.as_ref().unwrap().delete_secret(&name).await {
                    Ok(()) => {
                        self.set_status(&format!("Deleted secret: {name}"), false);
                        self.current_view = View::SecretsList;
                        self.previous_view = None;
                        self.load_secrets().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to delete: {e}"), true);
                        self.go_back();
                    }
                }
                self.is_loading = false;
            }
            ConfirmAction::DestroyVersion(secret_name, version) => {
                self.is_loading = true;
                match self
                    .client
                    .as_ref()
                    .unwrap()
                    .destroy_version(&secret_name, &version)
                    .await
                {
                    Ok(_) => {
                        self.set_status(&format!("Destroyed version: {version}"), false);
                        self.go_back();
                        self.load_versions().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to destroy: {e}"), true);
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
                match version.state {
                    VersionState::Destroyed => {
                        self.set_status(
                            "Cannot access destroyed version - data is permanently gone",
                            true,
                        );
                        return Ok(());
                    }
                    VersionState::Disabled => {
                        self.set_status("Version is disabled - press 'e' to enable it first", true);
                        return Ok(());
                    }
                    _ => {}
                }

                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();

                self.is_loading = true;
                match self
                    .client
                    .as_ref()
                    .unwrap()
                    .access_version(&secret_name, &version_num)
                    .await
                {
                    Ok(value) => {
                        self.revealed_value = Some(value);
                        self.set_status("Press 's' to hide value", false);
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to access: {e}"), true);
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
                match version.state {
                    VersionState::Destroyed => {
                        self.set_status(
                            "Cannot copy destroyed version - data is permanently gone",
                            true,
                        );
                        return Ok(());
                    }
                    VersionState::Disabled => {
                        self.set_status("Version is disabled - press 'e' to enable it first", true);
                        return Ok(());
                    }
                    _ => {}
                }

                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();

                self.is_loading = true;
                match self
                    .client
                    .as_ref()
                    .unwrap()
                    .access_version(&secret_name, &version_num)
                    .await
                {
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
                        self.set_status(&format!("Failed to access: {e}"), true);
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
                if version.state != VersionState::Disabled {
                    self.set_status("Can only enable disabled versions", true);
                    return Ok(());
                }

                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();

                self.is_loading = true;
                match self
                    .client
                    .as_ref()
                    .unwrap()
                    .enable_version(&secret_name, &version_num)
                    .await
                {
                    Ok(_) => {
                        self.set_status(&format!("Enabled version: {version_num}"), false);
                        self.load_versions().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to enable: {e}"), true);
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
                if version.state != VersionState::Enabled {
                    self.set_status("Can only disable enabled versions", true);
                    return Ok(());
                }

                let secret_name = secret.short_name.clone();
                let version_num = version.version.clone();

                self.is_loading = true;
                match self
                    .client
                    .as_ref()
                    .unwrap()
                    .disable_version(&secret_name, &version_num)
                    .await
                {
                    Ok(_) => {
                        self.set_status(&format!("Disabled version: {version_num}"), false);
                        self.load_versions().await?;
                    }
                    Err(e) => {
                        self.set_status(&format!("Failed to disable: {e}"), true);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_client::ReplicationPolicy;

    /// Helper to create a mock `SecretInfo` for testing.
    fn mock_secret(name: &str) -> SecretInfo {
        SecretInfo {
            short_name: name.to_string(),
            create_time: "2024-01-01".to_string(),
            labels: vec![],
            annotations: vec![],
            replication: ReplicationPolicy::Automatic,
            topics: vec![],
            version_aliases: vec![],
            rotation: None,
            version_destroy_ttl: None,
        }
    }

    // --- Constructor Tests ---

    #[test]
    fn test_new_with_project_id() {
        let app = App::new(Some("my-project".to_string()));

        assert_eq!(app.project_id, "my-project");
        assert_eq!(app.current_view, View::SecretsList);
        assert!(app.secrets.is_empty());
        assert!(app.client.is_none());
    }

    #[test]
    fn test_new_without_project_id() {
        let app = App::new(None);

        assert!(app.project_id.is_empty());
        assert_eq!(app.current_view, View::ProjectSelector);
    }

    // --- Input Buffer Edge Case ---

    #[test]
    fn test_input_backspace_on_empty() {
        let mut app = App::new(None);
        assert!(app.input_buffer.is_empty());

        // Should not panic on empty buffer
        app.input_backspace();
        assert!(app.input_buffer.is_empty());
    }

    // --- Navigation Tests ---

    #[test]
    fn test_select_next_secret_wraps() {
        let mut app = App::new(Some("test".to_string()));
        app.secrets = vec![mock_secret("a"), mock_secret("b"), mock_secret("c")];
        app.secrets_state.select(Some(2)); // Select last item

        app.select_next_secret();

        assert_eq!(app.secrets_state.selected(), Some(0)); // Wrapped to first
    }

    #[test]
    fn test_select_previous_secret_wraps() {
        let mut app = App::new(Some("test".to_string()));
        app.secrets = vec![mock_secret("a"), mock_secret("b"), mock_secret("c")];
        app.secrets_state.select(Some(0)); // Select first item

        app.select_previous_secret();

        assert_eq!(app.secrets_state.selected(), Some(2)); // Wrapped to last
    }

    #[test]
    fn test_navigation_on_empty_list() {
        let mut app = App::new(Some("test".to_string()));
        assert!(app.secrets.is_empty());

        // Should not panic on empty list
        app.select_next_secret();
        app.select_previous_secret();
        app.select_first_secret();
        app.select_last_secret();

        // Selection should remain None
        assert_eq!(app.secrets_state.selected(), None);
    }

    // --- Version Navigation Unique Behavior ---

    #[test]
    fn test_version_navigation_clears_revealed_value() {
        let mut app = App::new(Some("test".to_string()));
        app.versions = vec![
            VersionInfo {
                version: "1".to_string(),
                state: VersionState::Enabled,
                create_time: "2024-01-01".to_string(),
                destroy_time: None,
                scheduled_destroy_time: None,
                has_checksum: false,
            },
            VersionInfo {
                version: "2".to_string(),
                state: VersionState::Enabled,
                create_time: "2024-01-02".to_string(),
                destroy_time: None,
                scheduled_destroy_time: None,
                has_checksum: false,
            },
        ];
        app.versions_state.select(Some(0));
        app.revealed_value = Some("secret-value".to_string());

        app.select_next_version();

        assert!(app.revealed_value.is_none()); // Should be cleared
        assert_eq!(app.versions_state.selected(), Some(1));
    }

    // --- View Navigation Tests ---

    #[test]
    fn test_go_back_with_previous_view() {
        let mut app = App::new(Some("test".to_string()));
        app.current_view = View::SecretDetail;
        app.previous_view = Some(View::SecretsList);

        app.go_back();

        assert_eq!(app.current_view, View::SecretsList);
        assert!(app.previous_view.is_none()); // Should be consumed
    }

    #[test]
    fn test_go_back_defaults_to_secrets_list() {
        let mut app = App::new(Some("test".to_string()));
        app.current_view = View::SecretDetail;
        app.previous_view = None; // No previous view

        app.go_back();

        assert_eq!(app.current_view, View::SecretsList); // Default fallback
    }

    // --- Mode Transition Tests ---

    #[test]
    fn test_start_new_secret_clears_buffer() {
        let mut app = App::new(Some("test".to_string()));
        app.current_view = View::SecretsList;
        app.input_buffer = "leftover text".to_string();

        app.start_new_secret();

        assert!(app.input_buffer.is_empty()); // Buffer cleared
        assert_eq!(app.current_view, View::Input(InputMode::NewSecretName));
        assert_eq!(app.previous_view, Some(View::SecretsList)); // Saved for go_back
    }

    // --- Cursor Movement Tests ---

    #[test]
    fn test_cursor_left_moves_position() {
        let mut app = App::new(None);
        app.input_buffer = "hello".to_string();
        app.cursor_position = 5; // At the end

        app.cursor_left();
        assert_eq!(app.cursor_position, 4);

        app.cursor_left();
        assert_eq!(app.cursor_position, 3);
    }

    #[test]
    fn test_cursor_left_stops_at_beginning() {
        let mut app = App::new(None);
        app.input_buffer = "hello".to_string();
        app.cursor_position = 0; // At the beginning

        app.cursor_left();
        assert_eq!(app.cursor_position, 0); // Should not go negative
    }

    #[test]
    fn test_cursor_right_moves_position() {
        let mut app = App::new(None);
        app.input_buffer = "hello".to_string();
        app.cursor_position = 0; // At the beginning

        app.cursor_right();
        assert_eq!(app.cursor_position, 1);

        app.cursor_right();
        assert_eq!(app.cursor_position, 2);
    }

    #[test]
    fn test_cursor_right_stops_at_end() {
        let mut app = App::new(None);
        app.input_buffer = "hello".to_string();
        app.cursor_position = 5; // At the end

        app.cursor_right();
        assert_eq!(app.cursor_position, 5); // Should not go past the end
    }

    #[test]
    fn test_input_char_at_cursor_position() {
        let mut app = App::new(None);
        app.input_buffer = "hllo".to_string();
        app.cursor_position = 1; // After 'h'

        app.input_char('e');

        assert_eq!(app.input_buffer, "hello");
        assert_eq!(app.cursor_position, 2); // Cursor moves after inserted char
    }

    #[test]
    fn test_input_char_at_end() {
        let mut app = App::new(None);
        app.input_buffer = "hell".to_string();
        app.cursor_position = 4; // At the end

        app.input_char('o');

        assert_eq!(app.input_buffer, "hello");
        assert_eq!(app.cursor_position, 5);
    }

    #[test]
    fn test_input_backspace_at_cursor_position() {
        let mut app = App::new(None);
        app.input_buffer = "heello".to_string();
        app.cursor_position = 3; // After "hee"

        app.input_backspace();

        assert_eq!(app.input_buffer, "hello");
        assert_eq!(app.cursor_position, 2); // Cursor moves back
    }

    #[test]
    fn test_input_backspace_at_beginning_does_nothing() {
        let mut app = App::new(None);
        app.input_buffer = "hello".to_string();
        app.cursor_position = 0; // At the beginning

        app.input_backspace();

        assert_eq!(app.input_buffer, "hello"); // Unchanged
        assert_eq!(app.cursor_position, 0);
    }

    #[test]
    fn test_start_new_secret_resets_cursor() {
        let mut app = App::new(Some("test".to_string()));
        app.input_buffer = "some text".to_string();
        app.cursor_position = 5;

        app.start_new_secret();

        assert!(app.input_buffer.is_empty());
        assert_eq!(app.cursor_position, 0); // Cursor reset to beginning
    }
}

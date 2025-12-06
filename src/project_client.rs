//! Google Cloud Resource Manager client wrapper.
//!
//! This module provides a simple interface to list GCP projects
//! accessible to the authenticated user.

use anyhow::{Context, Result};
use google_cloud_resourcemanager_v3::client::Projects;

/// Information about a GCP project (simplified view).
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// The project ID (e.g., "my-project-123")
    pub project_id: String,
    /// The display name (human-readable name)
    pub display_name: String,
}

/// Fetches the list of projects accessible to the current user.
///
/// This uses Application Default Credentials (ADC) for authentication.
/// Make sure you have run: gcloud auth application-default login
pub async fn list_projects() -> Result<Vec<ProjectInfo>> {
    // Create the Resource Manager client
    let client = Projects::builder()
        .build()
        .await
        .context("Failed to create Resource Manager client")?;

    // Search for all projects the user has access to
    // An empty query returns all accessible projects
    let response = client
        .search_projects()
        .send()
        .await
        .context("Failed to list projects")?;

    // Convert to our simplified format
    let projects: Vec<ProjectInfo> = response
        .projects
        .into_iter()
        .map(|p| ProjectInfo {
            project_id: p.project_id.clone(),
            display_name: if p.display_name.is_empty() {
                p.project_id
            } else {
                p.display_name
            },
        })
        .collect();

    Ok(projects)
}

//! Google Cloud Secret Manager client wrapper.
//!
//! This module provides a simplified interface to the Secret Manager API.
//! It wraps the official Google Cloud Rust SDK.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use google_cloud_secretmanager_v1::client::SecretManagerService;
use google_cloud_secretmanager_v1::model::{
    Replication, Secret, SecretPayload, SecretVersion,
};

/// Information about a secret (simplified view).
#[derive(Debug, Clone)]
pub struct SecretInfo {
    /// Full resource name (e.g., "projects/my-project/secrets/my-secret")
    pub name: String,
    /// Short name (just the secret name without the full path)
    pub short_name: String,
    /// Creation time as a string
    pub create_time: String,
    /// Labels/tags on the secret
    pub labels: Vec<(String, String)>,
}

/// Information about a secret version.
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Full resource name
    pub name: String,
    /// Version number (e.g., "1", "2", "latest")
    pub version: String,
    /// State: ENABLED, DISABLED, or DESTROYED
    pub state: String,
    /// Creation time
    pub create_time: String,
}

/// Wrapper around the Google Cloud Secret Manager client.
pub struct SecretClient {
    /// The underlying Google Cloud client
    client: SecretManagerService,
    /// The Google Cloud project ID
    project_id: String,
}

impl SecretClient {
    /// Creates a new Secret Manager client for the given project.
    ///
    /// This will use Application Default Credentials (ADC) for authentication.
    /// Make sure you have run: gcloud auth application-default login
    pub async fn new(project_id: String) -> Result<Self> {
        let client = SecretManagerService::builder()
            .build()
            .await
            .context("Failed to create Secret Manager client. Make sure you have authenticated with: gcloud auth application-default login")?;

        Ok(Self { client, project_id })
    }

    /// Returns the parent path for API calls.
    fn parent(&self) -> String {
        format!("projects/{}", self.project_id)
    }

    /// Lists all secrets in the project.
    pub async fn list_secrets(&self) -> Result<Vec<SecretInfo>> {
        // Use the paginated list_secrets API
        let response = self
            .client
            .list_secrets()
            .set_parent(&self.parent())
            .send()
            .await
            .context("Failed to list secrets")?;

        // Convert secrets to our simplified format
        let secrets = response
            .secrets
            .into_iter()
            .map(|s| self.secret_to_info(&s))
            .collect();

        Ok(secrets)
    }

    /// Gets details for a specific secret.
    pub async fn get_secret(&self, secret_name: &str) -> Result<SecretInfo> {
        let name = self.secret_path(secret_name);
        let secret = self
            .client
            .get_secret()
            .set_name(&name)
            .send()
            .await
            .context("Failed to get secret")?;

        Ok(self.secret_to_info(&secret))
    }

    /// Lists all versions of a secret.
    pub async fn list_versions(&self, secret_name: &str) -> Result<Vec<VersionInfo>> {
        let parent = self.secret_path(secret_name);

        let response = self
            .client
            .list_secret_versions()
            .set_parent(&parent)
            .send()
            .await
            .context("Failed to list versions")?;

        let versions = response
            .versions
            .into_iter()
            .map(|v| self.version_to_info(&v))
            .collect();

        Ok(versions)
    }

    /// Gets the actual value of a secret version.
    ///
    /// This is the only way to retrieve the secret data.
    pub async fn access_version(&self, secret_name: &str, version: &str) -> Result<String> {
        let name = format!("{}/versions/{}", self.secret_path(secret_name), version);

        let response = self
            .client
            .access_secret_version()
            .set_name(&name)
            .send()
            .await
            .context("Failed to access secret version")?;

        // Extract the payload data
        let payload = response
            .payload
            .context("Secret version has no payload")?;

        // Convert bytes to string
        let data = payload.data;
        let value = String::from_utf8(data.into())
            .context("Secret value is not valid UTF-8")?;

        Ok(value)
    }

    /// Creates a new secret (without any version/value).
    pub async fn create_secret(&self, secret_name: &str) -> Result<SecretInfo> {
        // Set up automatic replication (Google manages the replication)
        let replication = Replication::default().set_automatic(
            google_cloud_secretmanager_v1::model::replication::Automatic::default(),
        );

        let secret = Secret::default().set_replication(replication);

        let created = self
            .client
            .create_secret()
            .set_parent(&self.parent())
            .set_secret_id(secret_name)
            .set_secret(secret)
            .send()
            .await
            .context("Failed to create secret")?;

        Ok(self.secret_to_info(&created))
    }

    /// Adds a new version to an existing secret.
    pub async fn add_version(&self, secret_name: &str, value: &str) -> Result<VersionInfo> {
        let parent = self.secret_path(secret_name);

        let payload = SecretPayload::default().set_data(value.as_bytes().to_vec());

        let version = self
            .client
            .add_secret_version()
            .set_parent(&parent)
            .set_payload(payload)
            .send()
            .await
            .context("Failed to add secret version")?;

        Ok(self.version_to_info(&version))
    }

    /// Enables a disabled secret version.
    pub async fn enable_version(&self, secret_name: &str, version: &str) -> Result<VersionInfo> {
        let name = format!("{}/versions/{}", self.secret_path(secret_name), version);

        let result = self
            .client
            .enable_secret_version()
            .set_name(&name)
            .send()
            .await
            .context("Failed to enable secret version")?;

        Ok(self.version_to_info(&result))
    }

    /// Disables an enabled secret version.
    pub async fn disable_version(&self, secret_name: &str, version: &str) -> Result<VersionInfo> {
        let name = format!("{}/versions/{}", self.secret_path(secret_name), version);

        let result = self
            .client
            .disable_secret_version()
            .set_name(&name)
            .send()
            .await
            .context("Failed to disable secret version")?;

        Ok(self.version_to_info(&result))
    }

    /// Destroys a secret version (irreversible!).
    pub async fn destroy_version(&self, secret_name: &str, version: &str) -> Result<VersionInfo> {
        let name = format!("{}/versions/{}", self.secret_path(secret_name), version);

        let result = self
            .client
            .destroy_secret_version()
            .set_name(&name)
            .send()
            .await
            .context("Failed to destroy secret version")?;

        Ok(self.version_to_info(&result))
    }

    /// Deletes a secret entirely (irreversible!).
    pub async fn delete_secret(&self, secret_name: &str) -> Result<()> {
        let name = self.secret_path(secret_name);

        self.client
            .delete_secret()
            .set_name(&name)
            .send()
            .await
            .context("Failed to delete secret")?;

        Ok(())
    }

    // --- Helper methods ---

    /// Returns the full path for a secret.
    fn secret_path(&self, secret_name: &str) -> String {
        format!("projects/{}/secrets/{}", self.project_id, secret_name)
    }

    /// Formats a protobuf timestamp as a date string (YYYY-MM-DD).
    fn format_timestamp(seconds: i64) -> String {
        DateTime::<Utc>::from_timestamp(seconds, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Converts a Secret proto to our SecretInfo struct.
    fn secret_to_info(&self, secret: &Secret) -> SecretInfo {
        let name = secret.name.clone();
        let short_name = name
            .rsplit('/')
            .next()
            .unwrap_or(&name)
            .to_string();

        let create_time = secret
            .create_time
            .as_ref()
            .map(|t| Self::format_timestamp(t.seconds()))
            .unwrap_or_else(|| "Unknown".to_string());

        let labels: Vec<(String, String)> = secret
            .labels
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        SecretInfo {
            name,
            short_name,
            create_time,
            labels,
        }
    }

    /// Converts a SecretVersion proto to our VersionInfo struct.
    fn version_to_info(&self, version: &SecretVersion) -> VersionInfo {
        let name = version.name.clone();

        // Extract version number from name (e.g., ".../versions/1" -> "1")
        let version_num = name
            .rsplit('/')
            .next()
            .unwrap_or("?")
            .to_string();

        let state = format!("{:?}", version.state);

        let create_time = version
            .create_time
            .as_ref()
            .map(|t| Self::format_timestamp(t.seconds()))
            .unwrap_or_else(|| "Unknown".to_string());

        VersionInfo {
            name,
            version: version_num,
            state,
            create_time,
        }
    }
}

//! Google Cloud Secret Manager client wrapper.
//!
//! This module provides a simplified interface to the Secret Manager API.
//! It wraps the official Google Cloud Rust SDK.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use google_cloud_secretmanager_v1::client::SecretManagerService;
use google_cloud_secretmanager_v1::model::{
    replication, secret_version, Replication, Secret, SecretPayload, SecretVersion,
};

/// Replication policy for a secret.
#[derive(Debug, Clone)]
pub enum ReplicationPolicy {
    /// Google manages replication automatically
    Automatic,
    /// User-managed replication with specific locations
    UserManaged(Vec<String>),
}

/// Rotation configuration for a secret.
#[derive(Debug, Clone)]
pub struct RotationConfig {
    /// Rotation period (e.g., "86400s" for 1 day)
    pub rotation_period: Option<String>,
    /// Next rotation time
    pub next_rotation_time: Option<String>,
}

/// Information about a secret (simplified view).
#[derive(Debug, Clone)]
pub struct SecretInfo {
    /// Short name (just the secret name without the full path)
    pub short_name: String,
    /// Creation time as a string
    pub create_time: String,
    /// Labels/tags on the secret
    pub labels: Vec<(String, String)>,
    /// Annotations (custom metadata)
    pub annotations: Vec<(String, String)>,
    /// Replication policy
    pub replication: ReplicationPolicy,
    /// Pub/Sub topics for notifications
    pub topics: Vec<String>,
    /// Version aliases (alias -> version number)
    pub version_aliases: Vec<(String, i64)>,
    /// Rotation configuration
    pub rotation: Option<RotationConfig>,
    /// Version destroy TTL (delayed destruction)
    pub version_destroy_ttl: Option<String>,
}

/// The state of a secret version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionState {
    /// Version is active and can be accessed
    Enabled,
    /// Version is disabled (can be re-enabled)
    Disabled,
    /// Version is permanently destroyed
    Destroyed,
    /// Unknown state (fallback)
    Unknown,
}

impl std::fmt::Display for VersionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Enabled => write!(f, "Enabled"),
            Self::Disabled => write!(f, "Disabled"),
            Self::Destroyed => write!(f, "Destroyed"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Information about a secret version.
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Version number (e.g., "1", "2", "latest")
    pub version: String,
    /// State of the version
    pub state: VersionState,
    /// Creation time
    pub create_time: String,
    /// Time when the version was destroyed (if applicable)
    pub destroy_time: Option<String>,
    /// Scheduled destruction time (for delayed destroy)
    pub scheduled_destroy_time: Option<String>,
    /// Whether a client-specified checksum was provided
    pub has_checksum: bool,
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
            .set_parent(self.parent())
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
    /// Returns the secret data as a string. If the data is not valid UTF-8,
    /// it returns a base64-encoded representation with a prefix indicator.
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
        let payload = response.payload.context("Secret version has no payload")?;

        // Try UTF-8 first, fall back to base64 for binary data
        let data: Vec<u8> = payload.data.into();
        if let Ok(value) = String::from_utf8(data.clone()) {
            Ok(value)
        } else {
            // Binary data - encode as base64 with indicator
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
            Ok(format!("[base64] {encoded}"))
        }
    }

    /// Creates a new secret (without any version/value).
    pub async fn create_secret(&self, secret_name: &str) -> Result<SecretInfo> {
        // Set up automatic replication (Google manages the replication)
        let replication = Replication::default()
            .set_automatic(google_cloud_secretmanager_v1::model::replication::Automatic::default());

        let secret = Secret::default().set_replication(replication);

        let created = self
            .client
            .create_secret()
            .set_parent(self.parent())
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
        DateTime::<Utc>::from_timestamp(seconds, 0).map_or_else(|| "Unknown".to_string(), |dt| dt.format("%Y-%m-%d").to_string())
    }

    /// Converts a Secret proto to our `SecretInfo` struct.
    fn secret_to_info(&self, secret: &Secret) -> SecretInfo {
        let short_name = secret
            .name
            .rsplit('/')
            .next()
            .unwrap_or(&secret.name)
            .to_string();

        let create_time = secret
            .create_time
            .as_ref().map_or_else(|| "Unknown".to_string(), |t| Self::format_timestamp(t.seconds()));

        let labels: Vec<(String, String)> = secret
            .labels
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let annotations: Vec<(String, String)> = secret
            .annotations
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Parse replication policy
        let replication = match &secret.replication {
            Some(r) => match &r.replication {
                Some(replication::Replication::UserManaged(um)) => {
                    let locations: Vec<String> = um
                        .replicas
                        .iter()
                        .map(|replica| replica.location.clone())
                        .collect();
                    ReplicationPolicy::UserManaged(locations)
                }
                _ => ReplicationPolicy::Automatic,
            },
            None => ReplicationPolicy::Automatic,
        };

        // Extract Pub/Sub topics
        let topics: Vec<String> = secret.topics.iter().map(|t| t.name.clone()).collect();

        // Extract version aliases
        let version_aliases: Vec<(String, i64)> = secret
            .version_aliases
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Parse rotation config
        let rotation = secret.rotation.as_ref().map(|r| RotationConfig {
            rotation_period: r
                .rotation_period
                .as_ref()
                .map(|d| format!("{}s", d.seconds())),
            next_rotation_time: r
                .next_rotation_time
                .as_ref()
                .map(|t| Self::format_timestamp(t.seconds())),
        });

        // Parse version destroy TTL
        let version_destroy_ttl = secret.version_destroy_ttl.as_ref().map(|d| {
            let secs = d.seconds();
            if secs >= 86400 {
                format!("{}d", secs / 86400)
            } else if secs >= 3600 {
                format!("{}h", secs / 3600)
            } else {
                format!("{secs}s")
            }
        });

        SecretInfo {
            short_name,
            create_time,
            labels,
            annotations,
            replication,
            topics,
            version_aliases,
            rotation,
            version_destroy_ttl,
        }
    }

    /// Converts a `SecretVersion` proto to our `VersionInfo` struct.
    fn version_to_info(&self, version: &SecretVersion) -> VersionInfo {
        // Extract version number from name (e.g., ".../versions/1" -> "1")
        let version_num = version.name.rsplit('/').next().unwrap_or("?").to_string();

        // Convert API state to our enum
        let state = match version.state {
            secret_version::State::Enabled => VersionState::Enabled,
            secret_version::State::Disabled => VersionState::Disabled,
            secret_version::State::Destroyed => VersionState::Destroyed,
            _ => VersionState::Unknown,
        };

        let create_time = version
            .create_time
            .as_ref().map_or_else(|| "Unknown".to_string(), |t| Self::format_timestamp(t.seconds()));

        let destroy_time = version
            .destroy_time
            .as_ref()
            .map(|t| Self::format_timestamp(t.seconds()));

        let scheduled_destroy_time = version
            .scheduled_destroy_time
            .as_ref()
            .map(|t| Self::format_timestamp(t.seconds()));

        VersionInfo {
            version: version_num,
            state,
            create_time,
            destroy_time,
            scheduled_destroy_time,
            has_checksum: version.client_specified_payload_checksum,
        }
    }
}

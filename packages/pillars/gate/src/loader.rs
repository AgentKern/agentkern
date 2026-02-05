//! Policy Loader
//!
//! Handles loading policies from filesystem or external sources.
//! Supports hot-reloading (future) and format detection (JSON/YAML).

use crate::policy::Policy;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Trait for loading policies.
#[async_trait::async_trait]
pub trait PolicyLoader: Send + Sync {
    /// Load all policies from the source.
    async fn load_all(&self) -> anyhow::Result<Vec<Policy>>;
}

/// File-based policy loader.
pub struct FilePolicyLoader {
    base_dir: PathBuf,
}

impl FilePolicyLoader {
    /// Create a new file loader watching the given directory.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Parse a single file.
    async fn parse_file(&self, path: &Path) -> anyhow::Result<Policy> {
        let content = fs::read_to_string(path).await?;
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let policy: Policy = match ext {
            "yaml" | "yml" => serde_yaml::from_str(&content)?,
            "json" => serde_json::from_str(&content)?,
            _ => anyhow::bail!("Unsupported file format: {}", ext),
        };

        Ok(policy)
    }
}

#[async_trait::async_trait]
impl PolicyLoader for FilePolicyLoader {
    async fn load_all(&self) -> anyhow::Result<Vec<Policy>> {
        let mut policies = Vec::new();

        if !self.base_dir.exists() {
            // It's okay if directory doesn't exist yet, just return empty
            tracing::warn!("Policy directory does not exist: {:?}", self.base_dir);
            return Ok(policies);
        }

        let mut entries = fs::read_dir(&self.base_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                match self.parse_file(&path).await {
                    Ok(policy) => {
                        tracing::info!("Loaded policy: {} ({})", policy.id, policy.name);
                        policies.push(policy);
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse policy {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(policies)
    }
}

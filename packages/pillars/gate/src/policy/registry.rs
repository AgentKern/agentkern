//! Shared Policy Registry for AgentKern Gate
//!
//! Manages importing and exporting of security policies in a standardized format.

use crate::policy::Policy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Metadata for a community-shared policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetadata {
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
    pub category: PolicyCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyCategory {
    Core,
    Community,
    Verified,
}

/// A policy bundled with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBundle {
    pub metadata: PolicyMetadata,
    pub policy: Policy,
}

pub struct PolicyRegistry {
    storage_path: PathBuf,
    policies: HashMap<String, PolicyBundle>,
}

impl PolicyRegistry {
    pub fn new(storage_path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = storage_path.into();
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create policy storage directory: {}", e))?;
        }
        Ok(Self {
            storage_path: path,
            policies: HashMap::new(),
        })
    }

    /// Load all policies from the storage directory.
    pub fn load_all(&mut self) -> Result<(), String> {
        let entries = fs::read_dir(&self.storage_path)
            .map_err(|e| format!("Failed to read policy directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path
                .extension()
                .is_some_and(|ext| ext == "json" || ext == "yaml")
            {
                match self.load_bundle(&path) {
                    Ok(bundle) => {
                        info!(
                            "Loaded policy bundle: {} (v{})",
                            bundle.policy.id, bundle.metadata.version
                        );
                        self.policies.insert(bundle.policy.id.clone(), bundle);
                    }
                    Err(e) => {
                        warn!("Failed to load policy bundle at {:?}: {}", path, e);
                    }
                }
            }
        }
        Ok(())
    }

    fn load_bundle(&self, path: &Path) -> Result<PolicyBundle, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        if path.extension().is_some_and(|ext| ext == "json") {
            serde_json::from_str(&content).map_err(|e| format!("JSON parse error: {}", e))
        } else {
            serde_yaml::from_str(&content).map_err(|e| format!("YAML parse error: {}", e))
        }
    }

    /// Save a policy bundle to the registry.
    pub fn save_bundle(&mut self, bundle: PolicyBundle) -> Result<(), String> {
        let filename = format!("{}.yaml", bundle.policy.id);
        let path = self.storage_path.join(filename);

        let content =
            serde_yaml::to_string(&bundle).map_err(|e| format!("Serialization error: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;

        self.policies.insert(bundle.policy.id.clone(), bundle);
        Ok(())
    }

    /// List all registered policies.
    pub fn list_policies(&self) -> Vec<&PolicyBundle> {
        self.policies.values().collect()
    }

    /// Get a policy bundle by ID.
    pub fn get_policy(&self, id: &str) -> Option<&PolicyBundle> {
        self.policies.get(id)
    }

    /// Export a policy as a standardized JSON bundle string.
    pub fn export_bundle(&self, id: &str) -> Result<String, String> {
        let bundle = self
            .get_policy(id)
            .ok_or_else(|| format!("Policy '{}' not found", id))?;

        serde_json::to_string_pretty(bundle)
            .map_err(|e| format!("Export serialization error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{Policy, PolicyAction, PolicyRule};
    use tempfile::tempdir;

    fn create_mock_bundle(id: &str) -> PolicyBundle {
        PolicyBundle {
            metadata: PolicyMetadata {
                author: "AgentKern Team".into(),
                version: "1.0.0".into(),
                tags: vec!["test".into()],
                category: PolicyCategory::Core,
            },
            policy: Policy {
                id: id.into(),
                name: format!("Test Policy {}", id),
                description: "Test description".into(),
                priority: 0,
                enabled: true,
                jurisdictions: vec![],
                namespace: "global".into(),
                rules: vec![PolicyRule {
                    id: "rule-1".into(),
                    condition: "true".into(),
                    action: PolicyAction::Allow,
                    message: None,
                    risk_score: None,
                }],
            },
        }
    }

    #[test]
    fn test_registry_save_and_load() {
        let dir = tempdir().unwrap();
        let mut registry = PolicyRegistry::new(dir.path()).unwrap();

        let bundle = create_mock_bundle("p1");
        registry.save_bundle(bundle).unwrap();

        let mut registry2 = PolicyRegistry::new(dir.path()).unwrap();
        registry2.load_all().unwrap();

        assert_eq!(registry2.list_policies().len(), 1);
        assert_eq!(
            registry2.get_policy("p1").unwrap().policy.name,
            "Test Policy p1"
        );
    }

    #[test]
    fn test_registry_export() {
        let dir = tempdir().unwrap();
        let mut registry = PolicyRegistry::new(dir.path()).unwrap();

        let bundle = create_mock_bundle("p1");
        registry.save_bundle(bundle).unwrap();

        let export = registry.export_bundle("p1").unwrap();
        assert!(export.contains("\"id\": \"p1\""));
    }
}

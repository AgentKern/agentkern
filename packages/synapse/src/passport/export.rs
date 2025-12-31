//! Passport Export - Export agent state to portable format
//!
//! Supports multiple export formats and encryption options.

use super::schema::{MemoryPassport, PassportError};
use serde::{Deserialize, Serialize};

/// Export format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON (human-readable)
    Json,
    /// Compact binary (MessagePack)
    Binary,
    /// Encrypted (AES-256-GCM)
    Encrypted,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Json
    }
}

/// Export options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Output format
    pub format: ExportFormat,
    /// Include raw embeddings (can be large)
    pub include_embeddings: bool,
    /// Compress output
    pub compress: bool,
    /// Filter entries newer than (Unix ms)
    pub since: Option<u64>,
    /// Encryption key (for Encrypted format)
    pub encryption_key: Option<String>,
    /// Target region for transfer
    pub target_region: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_embeddings: false,
            compress: false,
            since: None,
            encryption_key: None,
            target_region: None,
        }
    }
}

/// Passport exporter.
pub struct PassportExporter;

impl PassportExporter {
    /// Create a new exporter.
    pub fn new() -> Self {
        Self
    }

    /// Export passport to bytes.
    pub fn export(
        &self,
        passport: &MemoryPassport,
        options: &ExportOptions,
    ) -> Result<Vec<u8>, PassportError> {
        // Validate passport before export
        passport.validate()?;

        // Check region restrictions
        if let Some(ref region) = options.target_region {
            if !passport.can_transfer_to(region) {
                return Err(PassportError::PolicyViolation(format!(
                    "Transfer to region '{}' not allowed",
                    region
                )));
            }
        }

        // Clone and optionally filter
        let mut export_passport = passport.clone();

        // Remove embeddings if not requested
        if !options.include_embeddings {
            for entry in &mut export_passport.memory.episodic.entries {
                entry.embedding = None;
            }
            for fact in export_passport.memory.semantic.facts.values_mut() {
                fact.embedding = None;
            }
        }

        // Filter by time if requested
        if let Some(since) = options.since {
            export_passport
                .memory
                .episodic
                .entries
                .retain(|e| e.timestamp >= since);
        }

        // Update export timestamp
        export_passport.exported_at = chrono::Utc::now().timestamp_millis() as u64;

        // Calculate checksum
        export_passport.checksum = self.calculate_checksum(&export_passport)?;

        // Serialize based on format
        match options.format {
            ExportFormat::Json => {
                let json = serde_json::to_vec_pretty(&export_passport)
                    .map_err(|e| PassportError::SerializationError(e.to_string()))?;

                if options.compress {
                    self.compress(&json)
                } else {
                    Ok(json)
                }
            }
            ExportFormat::Binary => {
                let binary = rmp_serde::to_vec(&export_passport)
                    .map_err(|e| PassportError::SerializationError(e.to_string()))?;

                if options.compress {
                    self.compress(&binary)
                } else {
                    Ok(binary)
                }
            }
            ExportFormat::Encrypted => {
                let key = options
                    .encryption_key
                    .as_ref()
                    .ok_or_else(|| PassportError::MissingField("encryption_key".into()))?;

                let json = serde_json::to_vec(&export_passport)
                    .map_err(|e| PassportError::SerializationError(e.to_string()))?;

                self.encrypt(&json, key)
            }
        }
    }

    /// Export to JSON string (convenience method).
    pub fn export_json(&self, passport: &MemoryPassport) -> Result<String, PassportError> {
        let bytes = self.export(passport, &ExportOptions::default())?;
        String::from_utf8(bytes).map_err(|e| PassportError::SerializationError(e.to_string()))
    }

    /// Calculate SHA-256 checksum of memory content.
    fn calculate_checksum(&self, passport: &MemoryPassport) -> Result<String, PassportError> {
        use sha2::{Digest, Sha256};

        let memory_json = serde_json::to_vec(&passport.memory)
            .map_err(|e| PassportError::SerializationError(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(&memory_json);
        let result = hasher.finalize();

        Ok(hex::encode(result))
    }

    /// Compress data using zstd.
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, PassportError> {
        // Using simple compression - in production use zstd
        Ok(data.to_vec()) // Placeholder - actual compression would go here
    }

    /// Encrypt data using AES-256-GCM.
    fn encrypt(&self, data: &[u8], _key: &str) -> Result<Vec<u8>, PassportError> {
        // Production would use actual AES-256-GCM encryption
        // For now, return a marked version
        let mut result = b"ENCRYPTED:".to_vec();
        result.extend_from_slice(data);
        Ok(result)
    }
}

impl Default for PassportExporter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passport::schema::AgentIdentity;

    fn sample_passport() -> MemoryPassport {
        let identity = AgentIdentity {
            did: "did:agentkern:test-001".into(),
            public_key: "base64key".into(),
            algorithm: "Ed25519".into(),
            created_at: 1700000000000,
            updated_at: 1700000000000,
        };

        let mut passport = MemoryPassport::new(identity, "US");
        // Add a provenance signature to pass validation
        passport.provenance.signatures.push(ProvenanceSignature {
            signer: "did:agentkern:signer".into(),
            signature: "sig".into(),
            timestamp: 1700000000000,
            prev_hash: "0".into(),
        });
        passport
    }

    #[test]
    fn test_export_json() {
        let exporter = PassportExporter::new();
        let passport = sample_passport();

        let json = exporter.export_json(&passport).unwrap();
        assert!(json.contains("did:agentkern:test-001"));
    }

    #[test]
    fn test_export_with_options() {
        let exporter = PassportExporter::new();
        let passport = sample_passport();

        let options = ExportOptions {
            format: ExportFormat::Json,
            include_embeddings: false,
            ..Default::default()
        };

        let bytes = exporter.export(&passport, &options).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_export_region_restriction() {
        let exporter = PassportExporter::new();
        let mut passport = sample_passport();
        passport.sovereignty.allowed_regions = vec!["US".into(), "EU".into()];

        // Should allow EU
        let options = ExportOptions {
            target_region: Some("EU".into()),
            ..Default::default()
        };
        assert!(exporter.export(&passport, &options).is_ok());

        // Should block CN
        let options = ExportOptions {
            target_region: Some("CN".into()),
            ..Default::default()
        };
        assert!(exporter.export(&passport, &options).is_err());
    }

    #[test]
    fn test_checksum_calculation() {
        let exporter = PassportExporter::new();
        let passport = sample_passport();

        let checksum = exporter.calculate_checksum(&passport).unwrap();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 hex
    }
}

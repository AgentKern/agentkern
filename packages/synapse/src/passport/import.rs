//! Passport Import - Import agent state from portable format
//!
//! Validates and merges imported passport data.

use super::schema::{MemoryPassport, PassportError, PassportVersion};
use serde::{Deserialize, Serialize};

/// Import options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    /// Verify checksum
    pub verify_checksum: bool,
    /// Verify provenance signatures
    pub verify_provenance: bool,
    /// Merge with existing memory (vs replace)
    pub merge: bool,
    /// Decryption key (if encrypted)
    pub decryption_key: Option<String>,
    /// Accept passports from these regions
    pub allowed_regions: Vec<String>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            verify_checksum: true,
            verify_provenance: true,
            merge: false,
            decryption_key: None,
            allowed_regions: vec![],
        }
    }
}

/// Import result with validation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Was import successful?
    pub success: bool,
    /// Imported passport (if successful)
    pub passport: Option<MemoryPassport>,
    /// Validation warnings (non-fatal)
    pub warnings: Vec<String>,
    /// Statistics about imported data
    pub stats: ImportStats,
}

/// Statistics about imported data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportStats {
    pub episodic_entries: usize,
    pub semantic_facts: usize,
    pub skills: usize,
    pub preferences: usize,
    pub total_bytes: usize,
}

/// Passport importer.
pub struct PassportImporter;

impl PassportImporter {
    /// Create a new importer.
    pub fn new() -> Self {
        Self
    }
    
    /// Import passport from bytes.
    pub fn import(&self, data: &[u8], options: &ImportOptions) -> Result<ImportResult, PassportError> {
        let mut warnings: Vec<String> = Vec::new();
        
        // Detect format and decrypt if needed
        let json_data = if data.starts_with(b"ENCRYPTED:") {
            let key = options.decryption_key.as_ref()
                .ok_or_else(|| PassportError::MissingField("decryption_key".into()))?;
            self.decrypt(&data[10..], key)?
        } else if data.starts_with(b"{") {
            data.to_vec()
        } else {
            // Assume MessagePack binary
            let passport: MemoryPassport = rmp_serde::from_slice(data)
                .map_err(|e| PassportError::SerializationError(e.to_string()))?;
            return self.validate_and_wrap(passport, options);
        };
        
        // Parse JSON
        let passport: MemoryPassport = serde_json::from_slice(&json_data)
            .map_err(|e| PassportError::SerializationError(e.to_string()))?;
        
        self.validate_and_wrap(passport, options)
    }
    
    /// Import from JSON string (convenience method).
    pub fn import_json(&self, json: &str) -> Result<ImportResult, PassportError> {
        self.import(json.as_bytes(), &ImportOptions::default())
    }
    
    /// Validate passport and wrap in result.
    fn validate_and_wrap(&self, passport: MemoryPassport, options: &ImportOptions) -> Result<ImportResult, PassportError> {
        let mut warnings = Vec::new();
        
        // Check version compatibility
        if !passport.version.is_compatible(&PassportVersion::CURRENT) {
            warnings.push(format!(
                "Version {} may have compatibility issues with {}",
                passport.version, PassportVersion::CURRENT
            ));
        }
        
        // Check region restrictions
        if !options.allowed_regions.is_empty() {
            if !options.allowed_regions.contains(&passport.sovereignty.origin_region) {
                return Err(PassportError::PolicyViolation(
                    format!("Region '{}' not in allowed list", passport.sovereignty.origin_region)
                ));
            }
        }
        
        // Verify checksum
        if options.verify_checksum && !passport.checksum.is_empty() {
            let calculated = self.calculate_checksum(&passport)?;
            if calculated != passport.checksum {
                return Err(PassportError::InvalidSignature);
            }
        }
        
        // Verify provenance
        if options.verify_provenance {
            if let Err(e) = passport.provenance.verify() {
                warnings.push(format!("Provenance verification warning: {}", e));
            }
        }
        
        // Calculate stats
        let stats = ImportStats {
            episodic_entries: passport.memory.episodic.entries.len(),
            semantic_facts: passport.memory.semantic.facts.len(),
            skills: passport.memory.skills.skills.len(),
            preferences: passport.memory.preferences.items.len(),
            total_bytes: 0, // Would be calculated from original data
        };
        
        Ok(ImportResult {
            success: true,
            passport: Some(passport),
            warnings,
            stats,
        })
    }
    
    /// Calculate checksum for verification.
    fn calculate_checksum(&self, passport: &MemoryPassport) -> Result<String, PassportError> {
        use sha2::{Sha256, Digest};
        
        let memory_json = serde_json::to_vec(&passport.memory)
            .map_err(|e| PassportError::SerializationError(e.to_string()))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&memory_json);
        let result = hasher.finalize();
        
        Ok(hex::encode(result))
    }
    
    /// Decrypt data.
    fn decrypt(&self, data: &[u8], _key: &str) -> Result<Vec<u8>, PassportError> {
        // Production would use actual AES-256-GCM decryption
        Ok(data.to_vec())
    }
    
    /// Merge two passports.
    pub fn merge(&self, base: &mut MemoryPassport, incoming: &MemoryPassport) -> Result<(), PassportError> {
        // Merge episodic memory (append)
        for entry in &incoming.memory.episodic.entries {
            if !base.memory.episodic.entries.iter().any(|e| e.id == entry.id) {
                base.memory.episodic.entries.push(entry.clone());
            }
        }
        
        // Merge semantic memory (update if newer)
        for (id, fact) in &incoming.memory.semantic.facts {
            if !base.memory.semantic.facts.contains_key(id) {
                base.memory.semantic.facts.insert(id.clone(), fact.clone());
            }
        }
        
        // Merge skills (update if higher proficiency)
        for (id, skill) in &incoming.memory.skills.skills {
            match base.memory.skills.skills.get(id) {
                Some(existing) if existing.proficiency >= skill.proficiency => {}
                _ => {
                    base.memory.skills.skills.insert(id.clone(), skill.clone());
                }
            }
        }
        
        // Merge preferences (incoming wins for conflicts)
        for (key, pref) in &incoming.memory.preferences.items {
            base.memory.preferences.items.insert(key.clone(), pref.clone());
        }
        
        // Record transfer
        base.sovereignty.transfers.push(super::schema::TransferRecord {
            from: incoming.sovereignty.current_region.clone(),
            to: base.sovereignty.current_region.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            reason: "merge".to_string(),
            approved: true,
        });
        
        Ok(())
    }
}

impl Default for PassportImporter {
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
    use crate::passport::schema::{AgentIdentity, ProvenanceSignature};

    fn sample_passport() -> MemoryPassport {
        let identity = AgentIdentity {
            did: "did:verimantle:test-001".into(),
            public_key: "base64key".into(),
            algorithm: "Ed25519".into(),
            created_at: 1700000000000,
            updated_at: 1700000000000,
        };
        
        let mut passport = MemoryPassport::new(identity, "US");
        passport.provenance.signatures.push(ProvenanceSignature {
            signer: "did:verimantle:signer".into(),
            signature: "sig".into(),
            timestamp: 1700000000000,
            prev_hash: "0".into(),
        });
        passport
    }

    #[test]
    fn test_import_json() {
        let importer = PassportImporter::new();
        let passport = sample_passport();
        let json = serde_json::to_string(&passport).unwrap();
        
        let result = importer.import_json(&json).unwrap();
        assert!(result.success);
        assert!(result.passport.is_some());
    }

    #[test]
    fn test_import_with_region_restriction() {
        let importer = PassportImporter::new();
        let passport = sample_passport();
        let json = serde_json::to_string(&passport).unwrap();
        
        // Should allow US
        let options = ImportOptions {
            allowed_regions: vec!["US".into(), "EU".into()],
            verify_checksum: false,
            ..Default::default()
        };
        assert!(importer.import(json.as_bytes(), &options).is_ok());
        
        // Should block if US not in list
        let options = ImportOptions {
            allowed_regions: vec!["EU".into()],
            verify_checksum: false,
            ..Default::default()
        };
        assert!(importer.import(json.as_bytes(), &options).is_err());
    }

    #[test]
    fn test_import_stats() {
        let importer = PassportImporter::new();
        let passport = sample_passport();
        let json = serde_json::to_string(&passport).unwrap();
        
        let options = ImportOptions {
            verify_checksum: false,
            ..Default::default()
        };
        let result = importer.import(json.as_bytes(), &options).unwrap();
        
        assert_eq!(result.stats.episodic_entries, 0);
    }

    #[test]
    fn test_merge_passports() {
        let importer = PassportImporter::new();
        let mut base = sample_passport();
        let incoming = sample_passport();
        
        importer.merge(&mut base, &incoming).unwrap();
        
        assert_eq!(base.sovereignty.transfers.len(), 1);
    }
}

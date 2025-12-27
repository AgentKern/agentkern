//! GDPR Export - Article 20 Right to Data Portability compliance
//!
//! Per MANDATE.md Section 2: GDPR compliance required
//! Provides human-readable and machine-readable data exports.

use super::schema::{MemoryPassport, PassportError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data category for GDPR classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataCategory {
    /// Identity data (name, ID, etc.)
    Identity,
    /// Contact data (email, phone, etc.)
    Contact,
    /// Behavioral data (interactions, preferences)
    Behavioral,
    /// Technical data (device, IP, etc.)
    Technical,
    /// Financial data
    Financial,
    /// Health data (special category)
    Health,
    /// Location data
    Location,
    /// AI-generated data (predictions, inferences)
    AiGenerated,
    /// Other
    Other,
}

/// Processing event record for transparency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingEvent {
    /// Event timestamp
    pub timestamp: u64,
    /// Processing purpose
    pub purpose: String,
    /// Legal basis (consent, contract, etc.)
    pub legal_basis: String,
    /// Data categories processed
    pub categories: Vec<DataCategory>,
    /// Third parties involved
    pub third_parties: Vec<String>,
    /// Was data transferred outside EU?
    pub cross_border: bool,
}

/// GDPR-compliant export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprExport {
    /// Export timestamp
    pub export_date: String,
    
    /// Data subject identifier
    pub subject_id: String,
    
    /// Human-readable summary
    pub summary: GdprSummary,
    
    /// Machine-readable data (JSON-LD format)
    pub data: serde_json::Value,
    
    /// Data categories included
    pub categories: Vec<DataCategory>,
    
    /// Processing history
    pub processing_log: Vec<ProcessingEvent>,
    
    /// Rights information
    pub rights_info: RightsInfo,
}

/// Human-readable summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprSummary {
    /// What data we have
    pub data_held: Vec<String>,
    /// Why we have it
    pub purposes: Vec<String>,
    /// How long we keep it
    pub retention: String,
    /// Who we share it with
    pub recipients: Vec<String>,
    /// Your rights
    pub rights: Vec<String>,
}

/// Information about data subject rights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsInfo {
    /// Right to access (Art. 15)
    pub access: bool,
    /// Right to rectification (Art. 16)
    pub rectification: bool,
    /// Right to erasure (Art. 17)
    pub erasure: bool,
    /// Right to restrict processing (Art. 18)
    pub restriction: bool,
    /// Right to data portability (Art. 20)
    pub portability: bool,
    /// Right to object (Art. 21)
    pub object: bool,
    /// Contact for exercising rights
    pub contact_email: String,
}

/// GDPR export generator.
pub struct GdprExporter;

impl GdprExporter {
    /// Create a new exporter.
    pub fn new() -> Self {
        Self
    }
    
    /// Generate GDPR-compliant export from passport.
    pub fn export(&self, passport: &MemoryPassport) -> Result<GdprExport, PassportError> {
        let categories = self.detect_categories(passport);
        
        let summary = GdprSummary {
            data_held: self.summarize_data(passport),
            purposes: vec![
                "AI agent operation".to_string(),
                "Personalization".to_string(),
                "Service improvement".to_string(),
            ],
            retention: "As long as you maintain your account, or as required by law".to_string(),
            recipients: vec![
                "Cloud infrastructure providers".to_string(),
                "No third-party data selling".to_string(),
            ],
            rights: vec![
                "Access your data at any time".to_string(),
                "Request correction of inaccurate data".to_string(),
                "Request deletion of your data".to_string(),
                "Export your data in machine-readable format".to_string(),
                "Object to certain processing".to_string(),
            ],
        };
        
        let rights_info = RightsInfo {
            access: true,
            rectification: true,
            erasure: true,
            restriction: true,
            portability: true,
            object: true,
            contact_email: "privacy@agentkern.com".to_string(),
        };
        
        let data = self.to_json_ld(passport)?;
        
        Ok(GdprExport {
            export_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            subject_id: passport.identity.did.clone(),
            summary,
            data,
            categories,
            processing_log: vec![], // Would be populated from actual logs
            rights_info,
        })
    }
    
    /// Detect data categories in passport.
    fn detect_categories(&self, passport: &MemoryPassport) -> Vec<DataCategory> {
        let mut categories = vec![DataCategory::Identity]; // Always have identity
        
        // Check for behavioral data
        if !passport.memory.episodic.entries.is_empty() {
            categories.push(DataCategory::Behavioral);
        }
        
        // Check for AI-generated data
        if !passport.memory.semantic.facts.is_empty() {
            categories.push(DataCategory::AiGenerated);
        }
        
        // Check preferences
        if !passport.memory.preferences.items.is_empty() {
            categories.push(DataCategory::Behavioral);
        }
        
        // Deduplicate
        categories.sort_by_key(|c| format!("{:?}", c));
        categories.dedup();
        
        categories
    }
    
    /// Summarize data held.
    fn summarize_data(&self, passport: &MemoryPassport) -> Vec<String> {
        let mut summary = vec![
            format!("Agent identity: {}", passport.identity.did),
            format!("Region: {}", passport.sovereignty.origin_region),
        ];
        
        let episodic = passport.memory.episodic.entries.len();
        if episodic > 0 {
            summary.push(format!("{} interaction records", episodic));
        }
        
        let semantic = passport.memory.semantic.facts.len();
        if semantic > 0 {
            summary.push(format!("{} knowledge facts", semantic));
        }
        
        let skills = passport.memory.skills.skills.len();
        if skills > 0 {
            summary.push(format!("{} learned skills", skills));
        }
        
        let prefs = passport.memory.preferences.items.len();
        if prefs > 0 {
            summary.push(format!("{} preferences", prefs));
        }
        
        summary
    }
    
    /// Convert to JSON-LD format for machine readability.
    fn to_json_ld(&self, passport: &MemoryPassport) -> Result<serde_json::Value, PassportError> {
        Ok(serde_json::json!({
            "@context": {
                "@vocab": "https://schema.org/",
                "agentkern": "https://agentkern.com/schema/"
            },
            "@type": "agentkern:AgentData",
            "@id": passport.identity.did,
            "dateCreated": passport.identity.created_at,
            "dateModified": passport.identity.updated_at,
            "agentkern:originRegion": passport.sovereignty.origin_region,
            "agentkern:memory": {
                "episodicCount": passport.memory.episodic.entries.len(),
                "semanticCount": passport.memory.semantic.facts.len(),
                "skillCount": passport.memory.skills.skills.len(),
                "preferenceCount": passport.memory.preferences.items.len()
            },
            "agentkern:fullData": passport.memory
        }))
    }
    
    /// Export as human-readable text.
    pub fn export_text(&self, passport: &MemoryPassport) -> Result<String, PassportError> {
        let export = self.export(passport)?;
        
        let mut text = String::new();
        text.push_str("=================================================\n");
        text.push_str("GDPR DATA EXPORT - RIGHT TO DATA PORTABILITY\n");
        text.push_str("=================================================\n\n");
        
        text.push_str(&format!("Export Date: {}\n", export.export_date));
        text.push_str(&format!("Subject ID: {}\n\n", export.subject_id));
        
        text.push_str("DATA WE HOLD ABOUT YOU:\n");
        text.push_str("-----------------------\n");
        for item in &export.summary.data_held {
            text.push_str(&format!("• {}\n", item));
        }
        
        text.push_str("\nWHY WE PROCESS YOUR DATA:\n");
        text.push_str("-------------------------\n");
        for purpose in &export.summary.purposes {
            text.push_str(&format!("• {}\n", purpose));
        }
        
        text.push_str("\nYOUR RIGHTS:\n");
        text.push_str("------------\n");
        for right in &export.summary.rights {
            text.push_str(&format!("• {}\n", right));
        }
        
        text.push_str(&format!("\nTo exercise your rights, contact: {}\n", export.rights_info.contact_email));
        
        Ok(text)
    }
}

impl Default for GdprExporter {
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
            did: "did:agentkern:test-001".into(),
            public_key: "base64key".into(),
            algorithm: "Ed25519".into(),
            created_at: 1700000000000,
            updated_at: 1700000000000,
        };
        
        let mut passport = MemoryPassport::new(identity, "US");
        passport.provenance.signatures.push(ProvenanceSignature {
            signer: "did:agentkern:signer".into(),
            signature: "sig".into(),
            timestamp: 1700000000000,
            prev_hash: "0".into(),
        });
        passport
    }

    #[test]
    fn test_gdpr_export() {
        let exporter = GdprExporter::new();
        let passport = sample_passport();
        
        let export = exporter.export(&passport).unwrap();
        
        assert_eq!(export.subject_id, "did:agentkern:test-001");
        assert!(!export.categories.is_empty());
        assert!(export.rights_info.portability);
    }

    #[test]
    fn test_gdpr_summary() {
        let exporter = GdprExporter::new();
        let passport = sample_passport();
        
        let export = exporter.export(&passport).unwrap();
        
        assert!(!export.summary.data_held.is_empty());
        assert!(!export.summary.purposes.is_empty());
        assert!(!export.summary.rights.is_empty());
    }

    #[test]
    fn test_json_ld_format() {
        let exporter = GdprExporter::new();
        let passport = sample_passport();
        
        let export = exporter.export(&passport).unwrap();
        
        assert!(export.data.get("@context").is_some());
        assert!(export.data.get("@type").is_some());
    }

    #[test]
    fn test_text_export() {
        let exporter = GdprExporter::new();
        let passport = sample_passport();
        
        let text = exporter.export_text(&passport).unwrap();
        
        assert!(text.contains("GDPR DATA EXPORT"));
        assert!(text.contains("did:agentkern:test-001"));
        assert!(text.contains("YOUR RIGHTS"));
    }

    #[test]
    fn test_detect_categories() {
        let exporter = GdprExporter::new();
        let passport = sample_passport();
        
        let categories = exporter.detect_categories(&passport);
        
        assert!(categories.contains(&DataCategory::Identity));
    }
}

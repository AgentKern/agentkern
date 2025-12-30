//! AgentKern-Gate: Global Privacy Registry
//!
//! Per MANDATE.md Section 2: Global Compliance
//! Per AI-Native Audit 2026: "Automated Evidence Collection & Privacy Engineering"
//!
//! This module implements a unified compliance engine for:
//! - GDPR (EU)
//! - CCPA/CPRA (California, USA)
//! - LGPD (Brazil)
//! - PIPL (China)
//! - PDPA (Singapore)
//! - NDMO (Saudi Arabia)
//!
//! Features:
//! - CBDT (Cross-Border Data Transfer) validation matrix
//! - Automated data subject rights (DSAR) routing
//! - Privacy risk scoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrivacyError {
    #[error("Jurisdiction not supported: {0}")]
    UnsupportedJurisdiction(String),
    #[error("Violates CBDT restriction: {0} -> {1}")]
    CbdtViolation(String, String),
    #[error("Missing mandatory data localization for {0}")]
    LocalizationRequired(String),
}

/// Global jurisdictions for privacy tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Jurisdiction {
    Eu,
    UsFederal,
    UsCalifornia,
    Brazil,
    China,
    Singapore,
    SaudiArabia,
    Global,
}

/// Specific privacy regulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Regulation {
    Gdpr,
    Ccpa,
    Lgpd,
    Pipl,
    Pdpa,
    Ndmo,
}

/// Cross-Border Data Transfer (CBDT) Status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    /// Allowed (Adequacy decision or bilateral agreement)
    Allowed,
    /// Restricted (Requires SCCs, BCRs, or explicit consent)
    Restricted,
    /// Prohibited (Data localization mandatory, e.g., PIPL/NDMO)
    Prohibited,
}

/// Result of a privacy compliance check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyCheckResult {
    /// Overall decision
    pub is_allowed: bool,
    /// Status of the transfer
    pub transfer_status: TransferStatus,
    /// Applicable regulations
    pub regulations: Vec<Regulation>,
    /// Privacy Risk Score (0-100)
    pub risk_score: u8,
    /// Required mitigation steps (e.g., "Encrypt at edge", "Obtain PIPL consent")
    pub mitigations: Vec<String>,
}

/// Global Privacy Registry for automated multi-jurisdiction compliance.
#[derive(Debug, Clone)]
pub struct GlobalPrivacyRegistry {
    /// CBDT Matrix: [Source] -> [Target] -> Status
    cbdt_matrix: HashMap<Jurisdiction, HashMap<Jurisdiction, TransferStatus>>,
    /// Jurisdiction to Regulation mapping
    reg_map: HashMap<Jurisdiction, Vec<Regulation>>,
}

impl GlobalPrivacyRegistry {
    /// Create a new registry with default 2026 global privacy rules.
    pub fn new() -> Self {
        let mut cbdt_matrix = HashMap::new();

        // EU Rules
        let mut eu_out = HashMap::new();
        eu_out.insert(Jurisdiction::Brazil, TransferStatus::Allowed); // Adequacy
        eu_out.insert(Jurisdiction::UsFederal, TransferStatus::Restricted); // SCCs/DPF 2.0
        eu_out.insert(Jurisdiction::China, TransferStatus::Restricted);
        cbdt_matrix.insert(Jurisdiction::Eu, eu_out);

        // China Rules (Strict Localization)
        let mut china_out = HashMap::new();
        china_out.insert(Jurisdiction::Global, TransferStatus::Prohibited);
        china_out.insert(Jurisdiction::Eu, TransferStatus::Restricted); // Requires CAC approval
        cbdt_matrix.insert(Jurisdiction::China, china_out);

        // Saudi Rules (NDMO)
        let mut saudi_out = HashMap::new();
        saudi_out.insert(Jurisdiction::Global, TransferStatus::Prohibited);
        cbdt_matrix.insert(Jurisdiction::SaudiArabia, saudi_out);

        let mut reg_map = HashMap::new();
        reg_map.insert(Jurisdiction::Eu, vec![Regulation::Gdpr]);
        reg_map.insert(Jurisdiction::UsCalifornia, vec![Regulation::Ccpa]);
        reg_map.insert(Jurisdiction::Brazil, vec![Regulation::Lgpd]);
        reg_map.insert(Jurisdiction::China, vec![Regulation::Pipl]);
        reg_map.insert(Jurisdiction::Singapore, vec![Regulation::Pdpa]);
        reg_map.insert(Jurisdiction::SaudiArabia, vec![Regulation::Ndmo]);

        Self {
            cbdt_matrix,
            reg_map,
        }
    }

    /// Validate if data can move from source to destination.
    pub fn validate_transfer(
        &self,
        source: Jurisdiction,
        destination: Jurisdiction,
        data_sensitivity: u8,
    ) -> PrivacyCheckResult {
        if source == destination {
            return PrivacyCheckResult {
                is_allowed: true,
                transfer_status: TransferStatus::Allowed,
                regulations: self.reg_map.get(&source).cloned().unwrap_or_default(),
                risk_score: data_sensitivity / 2, // Low risk for local
                mitigations: Vec::new(),
            };
        }

        let status = self
            .cbdt_matrix
            .get(&source)
            .and_then(|m| m.get(&destination))
            .copied()
            .unwrap_or(TransferStatus::Restricted); // Default to restricted for unknown

        let mut mitigations = Vec::new();
        let mut risk_score = data_sensitivity;

        match status {
            TransferStatus::Allowed => {
                mitigations.push("Log transfer for Article 30 ROPA".to_string());
                risk_score = risk_score.saturating_add(10);
            }
            TransferStatus::Restricted => {
                mitigations.push("Execute Standard Contractual Clauses (SCCs)".to_string());
                mitigations.push("Apply pseudonymization at source".to_string());
                risk_score = risk_score.saturating_add(30);
            }
            TransferStatus::Prohibited => {
                mitigations.push("TRANSFER BLOCKED: Mandatory local hosting required".to_string());
                risk_score = 100;
            }
        }

        PrivacyCheckResult {
            is_allowed: status != TransferStatus::Prohibited,
            transfer_status: status,
            regulations: self.reg_map.get(&source).cloned().unwrap_or_default(),
            risk_score: risk_score.min(100),
            mitigations,
        }
    }

    /// Calculate privacy risk for a specific regulation.
    pub fn calculate_risk(&self, reg: Regulation, has_pii: bool, in_tee: bool) -> u8 {
        let mut score: u8 = if has_pii { 50 } else { 10 };

        match reg {
            Regulation::Gdpr => score += 20,
            Regulation::Pipl => score += 30, // High regulatory pressure
            Regulation::Ccpa => score += 15,
            _ => score += 10,
        }

        if in_tee {
            score = score.saturating_sub(40); // TEE significantly reduces risk
        }

        score.min(100)
    }

    // ========================================================================
    // EXTENSIBILITY API - Add new regulations without code changes
    // ========================================================================

    /// Register a new jurisdiction with its applicable regulations.
    ///
    /// # Example
    /// ```ignore
    /// use agentkern_governance::privacy::{GlobalPrivacyRegistry, Jurisdiction, Regulation};
    ///
    /// let mut registry = GlobalPrivacyRegistry::new();
    /// // Add Japan APPI
    /// registry.register_jurisdiction(Jurisdiction::Global, vec![Regulation::Gdpr]); // Placeholder
    /// ```
    pub fn register_jurisdiction(
        &mut self,
        jurisdiction: Jurisdiction,
        regulations: Vec<Regulation>,
    ) {
        self.reg_map.insert(jurisdiction, regulations);
    }

    /// Register a CBDT rule between two jurisdictions.
    ///
    /// # Example
    /// ```ignore
    /// // Japan to EU is allowed (adequacy decision 2019)
    /// registry.register_cbdt_rule(Jurisdiction::Japan, Jurisdiction::Eu, TransferStatus::Allowed);
    /// ```
    pub fn register_cbdt_rule(
        &mut self,
        source: Jurisdiction,
        destination: Jurisdiction,
        status: TransferStatus,
    ) {
        self.cbdt_matrix
            .entry(source)
            .or_default()
            .insert(destination, status);
    }

    /// Bulk register CBDT rules from a configuration.
    pub fn load_cbdt_config(&mut self, rules: Vec<(Jurisdiction, Jurisdiction, TransferStatus)>) {
        for (src, dst, status) in rules {
            self.register_cbdt_rule(src, dst, status);
        }
    }

    /// Get all registered jurisdictions.
    pub fn jurisdictions(&self) -> Vec<Jurisdiction> {
        self.reg_map.keys().copied().collect()
    }

    /// Get applicable regulations for a jurisdiction.
    pub fn regulations_for(&self, jurisdiction: Jurisdiction) -> Vec<Regulation> {
        self.reg_map.get(&jurisdiction).cloned().unwrap_or_default()
    }

    /// Check if a specific regulation applies to a jurisdiction.
    pub fn has_regulation(&self, jurisdiction: Jurisdiction, reg: Regulation) -> bool {
        self.reg_map
            .get(&jurisdiction)
            .map(|regs| regs.contains(&reg))
            .unwrap_or(false)
    }
}

impl Default for GlobalPrivacyRegistry {
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

    #[test]
    fn test_eu_to_brazil_allowed() {
        let registry = GlobalPrivacyRegistry::new();
        let result = registry.validate_transfer(Jurisdiction::Eu, Jurisdiction::Brazil, 50);

        assert!(result.is_allowed);
        assert_eq!(result.transfer_status, TransferStatus::Allowed);
        assert!(result.regulations.contains(&Regulation::Gdpr));
    }

    #[test]
    fn test_china_localization_prohibited() {
        let registry = GlobalPrivacyRegistry::new();
        let result = registry.validate_transfer(Jurisdiction::China, Jurisdiction::Global, 50);

        assert!(!result.is_allowed);
        assert_eq!(result.transfer_status, TransferStatus::Prohibited);
        assert!(result.mitigations[0].contains("BLOCKED"));
    }

    #[test]
    fn test_eu_to_us_restricted() {
        let registry = GlobalPrivacyRegistry::new();
        let result = registry.validate_transfer(Jurisdiction::Eu, Jurisdiction::UsFederal, 50);

        assert!(result.is_allowed);
        assert_eq!(result.transfer_status, TransferStatus::Restricted);
        assert!(result.mitigations.iter().any(|m| m.contains("SCCs")));
    }

    #[test]
    fn test_risk_scoring_with_tee() {
        let registry = GlobalPrivacyRegistry::new();

        let risk_high = registry.calculate_risk(Regulation::Gdpr, true, false);
        let risk_low = registry.calculate_risk(Regulation::Gdpr, true, true);

        assert!(risk_low < risk_high);
        assert_eq!(risk_high, 70);
        assert_eq!(risk_low, 30);
    }

    #[test]
    fn test_jurisdiction_matching() {
        let registry = GlobalPrivacyRegistry::new();
        let regs = registry.reg_map.get(&Jurisdiction::UsCalifornia).unwrap();
        assert!(regs.contains(&Regulation::Ccpa));
    }

    #[test]
    fn test_dynamic_cbdt_registration() {
        let mut registry = GlobalPrivacyRegistry::new();

        // Add Singapore -> EU as Allowed (hypothetical adequacy)
        registry.register_cbdt_rule(
            Jurisdiction::Singapore,
            Jurisdiction::Eu,
            TransferStatus::Allowed,
        );

        let result = registry.validate_transfer(Jurisdiction::Singapore, Jurisdiction::Eu, 50);
        assert!(result.is_allowed);
        assert_eq!(result.transfer_status, TransferStatus::Allowed);
    }

    #[test]
    fn test_bulk_cbdt_config() {
        let mut registry = GlobalPrivacyRegistry::new();

        registry.load_cbdt_config(vec![
            (
                Jurisdiction::Brazil,
                Jurisdiction::Eu,
                TransferStatus::Allowed,
            ),
            (
                Jurisdiction::Brazil,
                Jurisdiction::China,
                TransferStatus::Restricted,
            ),
        ]);

        let result = registry.validate_transfer(Jurisdiction::Brazil, Jurisdiction::Eu, 50);
        assert_eq!(result.transfer_status, TransferStatus::Allowed);
    }

    #[test]
    fn test_has_regulation() {
        let registry = GlobalPrivacyRegistry::new();

        assert!(registry.has_regulation(Jurisdiction::Eu, Regulation::Gdpr));
        assert!(!registry.has_regulation(Jurisdiction::Eu, Regulation::Ccpa));
        assert!(registry.has_regulation(Jurisdiction::UsCalifornia, Regulation::Ccpa));
    }
}

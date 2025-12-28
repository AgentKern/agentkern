//! AgentKern-Gate: Takaful (Islamic Insurance) Compliance
//!
//! Per EXECUTION_MANDATE.md §2: "Takaful (Islamic Insurance): Full support for compliant workflows"
//!
//! Features:
//! - Shariah-compliant workflow validation
//! - Interest (Riba) detection
//! - Gharar (uncertainty) risk assessment
//! - Takaful pool logic vs conventional insurance
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::takaful::{TakafulValidator, TransactionType};
//!
//! let validator = TakafulValidator::new();
//! let result = validator.validate_transaction(TransactionType::Insurance)?;
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Takaful compliance error.
#[derive(Debug, Error)]
pub enum TakafulError {
    #[error("Riba (interest) detected in transaction")]
    RibaDetected,
    #[error("Gharar (excessive uncertainty) detected")]
    GhararDetected,
    #[error("Maysir (gambling) element detected")]
    MaysirDetected,
    #[error("Transaction not Shariah-compliant: {reason}")]
    NotShariaCompliant { reason: String },
}

/// Type of financial transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Standard insurance (conventional)
    Insurance,
    /// Takaful (Islamic insurance)
    Takaful,
    /// Loan with interest
    Loan,
    /// Murabaha (cost-plus financing)
    Murabaha,
    /// Musharakah (partnership)
    Musharakah,
    /// Ijara (leasing)
    Ijara,
    /// General trade
    Trade,
}

/// Takaful compliance result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    /// Is transaction Shariah-compliant?
    pub compliant: bool,
    /// Compliance score (0-100)
    pub score: u8,
    /// Risk level for Gharar
    pub gharar_risk: RiskLevel,
    /// Contains Riba (interest)?
    pub has_riba: bool,
    /// Contains Maysir (gambling)?
    pub has_maysir: bool,
    /// Recommendations for compliance
    pub recommendations: Vec<String>,
}

/// Risk level enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Transaction details for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetails {
    /// Transaction type
    pub transaction_type: TransactionType,
    /// Amount in local currency
    pub amount: f64,
    /// Interest rate (if any)
    pub interest_rate: Option<f64>,
    /// Profit margin (for Murabaha)
    pub profit_margin: Option<f64>,
    /// Is outcome guaranteed?
    pub guaranteed_outcome: bool,
    /// Risk sharing percentage
    pub risk_sharing_pct: f64,
    /// Underlying asset present?
    pub has_underlying_asset: bool,
}

impl Default for TransactionDetails {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::Trade,
            amount: 0.0,
            interest_rate: None,
            profit_margin: None,
            guaranteed_outcome: false,
            risk_sharing_pct: 0.0,
            has_underlying_asset: true,
        }
    }
}

/// Takaful compliance validator.
#[derive(Debug, Default)]
pub struct TakafulValidator {
    /// Strict mode (reject any non-compliant transaction)
    strict_mode: bool,
}

impl TakafulValidator {
    /// Create a new validator.
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    /// Create a validator in strict mode.
    pub fn strict() -> Self {
        Self { strict_mode: true }
    }

    /// Validate a transaction for Shariah compliance.
    pub fn validate(&self, details: &TransactionDetails) -> Result<ComplianceResult, TakafulError> {
        let mut result = ComplianceResult {
            compliant: true,
            score: 100,
            gharar_risk: RiskLevel::Low,
            has_riba: false,
            has_maysir: false,
            recommendations: vec![],
        };

        // Check for Riba (interest)
        if let Some(rate) = details.interest_rate {
            if rate > 0.0 {
                result.has_riba = true;
                result.compliant = false;
                result.score = result.score.saturating_sub(50);
                result.recommendations.push(
                    "Replace interest-based financing with Murabaha (cost-plus) or Musharakah (profit-sharing)".to_string()
                );

                if self.strict_mode {
                    return Err(TakafulError::RibaDetected);
                }
            }
        }

        // Check for Gharar (excessive uncertainty)
        if !details.has_underlying_asset {
            result.gharar_risk = RiskLevel::High;
            result.score = result.score.saturating_sub(20);
            result
                .recommendations
                .push("Ensure transaction has a tangible underlying asset".to_string());
        }

        // Check for Maysir (gambling)
        if details.guaranteed_outcome && details.transaction_type == TransactionType::Insurance {
            result.has_maysir = true;
            result.score = result.score.saturating_sub(30);
            result
                .recommendations
                .push("Convert to Takaful model with mutual risk sharing".to_string());

            if self.strict_mode {
                return Err(TakafulError::MaysirDetected);
            }
        }

        // Check risk sharing for Islamic finance
        match details.transaction_type {
            TransactionType::Takaful | TransactionType::Musharakah => {
                if details.risk_sharing_pct < 50.0 {
                    result.score = result.score.saturating_sub(10);
                    result
                        .recommendations
                        .push("Increase risk sharing ratio for better compliance".to_string());
                }
            }
            TransactionType::Murabaha => {
                if details.profit_margin.unwrap_or(0.0) > 30.0 {
                    result.score = result.score.saturating_sub(10);
                    result.gharar_risk = RiskLevel::Medium;
                    result.recommendations.push(
                        "Consider reducing profit margin to align with market rates".to_string(),
                    );
                }
            }
            _ => {}
        }

        // Update compliance status
        result.compliant = result.score >= 70 && !result.has_riba;

        Ok(result)
    }

    /// Convert conventional insurance to Takaful model.
    pub fn convert_to_takaful(&self, details: &TransactionDetails) -> TransactionDetails {
        TransactionDetails {
            transaction_type: TransactionType::Takaful,
            amount: details.amount,
            interest_rate: None,       // Remove interest
            profit_margin: Some(10.0), // Standard Takaful margin
            guaranteed_outcome: false,
            risk_sharing_pct: 100.0, // Full mutual risk sharing
            has_underlying_asset: true,
        }
    }

    /// Check if a transaction type is inherently Shariah-compliant.
    pub fn is_compliant_type(&self, tx_type: TransactionType) -> bool {
        matches!(
            tx_type,
            TransactionType::Takaful
                | TransactionType::Murabaha
                | TransactionType::Musharakah
                | TransactionType::Ijara
                | TransactionType::Trade
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riba_detection() {
        let validator = TakafulValidator::new();
        let details = TransactionDetails {
            transaction_type: TransactionType::Loan,
            interest_rate: Some(5.0),
            ..Default::default()
        };

        let result = validator.validate(&details).unwrap();
        assert!(result.has_riba);
        assert!(!result.compliant);
    }

    #[test]
    fn test_takaful_compliance() {
        let validator = TakafulValidator::new();
        let details = TransactionDetails {
            transaction_type: TransactionType::Takaful,
            amount: 10000.0,
            interest_rate: None,
            profit_margin: Some(10.0),
            guaranteed_outcome: false,
            risk_sharing_pct: 100.0,
            has_underlying_asset: true,
        };

        let result = validator.validate(&details).unwrap();
        assert!(result.compliant);
        assert!(!result.has_riba);
        assert_eq!(result.score, 100);
    }

    #[test]
    fn test_strict_mode() {
        let validator = TakafulValidator::strict();
        let details = TransactionDetails {
            interest_rate: Some(5.0),
            ..Default::default()
        };

        let result = validator.validate(&details);
        assert!(matches!(result, Err(TakafulError::RibaDetected)));
    }

    #[test]
    fn test_convert_to_takaful() {
        let validator = TakafulValidator::new();
        let conventional = TransactionDetails {
            transaction_type: TransactionType::Insurance,
            amount: 5000.0,
            interest_rate: Some(3.0),
            guaranteed_outcome: true,
            risk_sharing_pct: 0.0,
            ..Default::default()
        };

        let takaful = validator.convert_to_takaful(&conventional);
        assert_eq!(takaful.transaction_type, TransactionType::Takaful);
        assert!(takaful.interest_rate.is_none());
        assert_eq!(takaful.risk_sharing_pct, 100.0);
    }

    #[test]
    fn test_compliant_types() {
        let validator = TakafulValidator::new();

        assert!(validator.is_compliant_type(TransactionType::Takaful));
        assert!(validator.is_compliant_type(TransactionType::Murabaha));
        assert!(!validator.is_compliant_type(TransactionType::Insurance));
        assert!(!validator.is_compliant_type(TransactionType::Loan));
    }
}

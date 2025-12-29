//! Shariah Compliance Checker
//!
//! Implements Islamic finance principles for agent operations:
//! - Riba (interest) prohibition
//! - Gharar (excessive uncertainty) prohibition  
//! - Maysir (gambling/speculation) prohibition
//! - Halal sector screening

use serde::{Deserialize, Serialize};

/// Shariah compliance checker.
pub struct ShariahCompliance {
    /// Enabled checks
    checks: Vec<ShariahCheck>,
    /// Strictness level
    strictness: StrictnessLevel,
}

/// Shariah check types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShariahCheck {
    /// No interest (riba)
    Riba,
    /// No excessive uncertainty (gharar)
    Gharar,
    /// No gambling/speculation (maysir)
    Maysir,
    /// Halal sector screening
    HalalSector,
    /// Purification of income
    Purification,
    /// Zakat calculation
    Zakat,
}

/// Strictness level for compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrictnessLevel {
    /// Lenient (mainstream scholars)
    Lenient,
    /// Moderate (AAOIFI standards)
    Moderate,
    /// Strict (conservative interpretation)
    Strict,
}

/// Shariah violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShariahViolation {
    /// Type of violation
    pub check: ShariahCheck,
    /// Severity (0.0 = minor, 1.0 = major)
    pub severity: f64,
    /// Description
    pub description: String,
    /// Remediation suggestion
    pub remediation: Option<String>,
}

/// Transaction to check for compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCheck {
    /// Transaction type
    pub transaction_type: TransactionType,
    /// Amount
    pub amount: u64,
    /// Currency
    pub currency: String,
    /// Counterparty sector (if known)
    pub sector: Option<String>,
    /// Interest rate (if any)
    pub interest_rate: Option<f64>,
    /// Uncertainty level (0.0 = certain, 1.0 = highly uncertain)
    pub uncertainty_level: Option<f64>,
}

/// Transaction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Payment,
    Investment,
    Loan,
    Trade,
    Insurance,
    Derivative,
}

impl ShariahCompliance {
    /// Create new compliance checker with default settings.
    pub fn new() -> Self {
        Self {
            checks: vec![
                ShariahCheck::Riba,
                ShariahCheck::Gharar,
                ShariahCheck::Maysir,
                ShariahCheck::HalalSector,
            ],
            strictness: StrictnessLevel::Moderate,
        }
    }
    
    /// Set strictness level.
    pub fn with_strictness(mut self, level: StrictnessLevel) -> Self {
        self.strictness = level;
        self
    }
    
    /// Check transaction for Shariah compliance.
    pub fn check_transaction(&self, tx: &TransactionCheck) -> ComplianceResult {
        let mut violations = Vec::new();
        
        // Check for Riba (interest)
        if self.checks.contains(&ShariahCheck::Riba) {
            if let Some(rate) = tx.interest_rate {
                if rate > 0.0 {
                    violations.push(ShariahViolation {
                        check: ShariahCheck::Riba,
                        severity: 1.0, // Major violation
                        description: format!("Transaction contains interest at {}%", rate * 100.0),
                        remediation: Some("Convert to profit-sharing (Mudarabah) or fee-based structure".into()),
                    });
                }
            }
        }
        
        // Check for Gharar (uncertainty)
        if self.checks.contains(&ShariahCheck::Gharar) {
            if let Some(uncertainty) = tx.uncertainty_level {
                let threshold = match self.strictness {
                    StrictnessLevel::Lenient => 0.8,
                    StrictnessLevel::Moderate => 0.6,
                    StrictnessLevel::Strict => 0.4,
                };
                
                if uncertainty > threshold {
                    violations.push(ShariahViolation {
                        check: ShariahCheck::Gharar,
                        severity: uncertainty,
                        description: "Excessive uncertainty in transaction".into(),
                        remediation: Some("Add clearer terms and reduce ambiguity".into()),
                    });
                }
            }
        }
        
        // Check for Maysir (gambling)
        if self.checks.contains(&ShariahCheck::Maysir) {
            if tx.transaction_type == TransactionType::Derivative {
                violations.push(ShariahViolation {
                    check: ShariahCheck::Maysir,
                    severity: 0.9,
                    description: "Derivatives are considered speculative".into(),
                    remediation: Some("Use Salam or Istisna contracts instead".into()),
                });
            }
        }
        
        // Check for Haram sectors
        if self.checks.contains(&ShariahCheck::HalalSector) {
            if let Some(sector) = &tx.sector {
                if self.is_prohibited_sector(sector) {
                    violations.push(ShariahViolation {
                        check: ShariahCheck::HalalSector,
                        severity: 1.0,
                        description: format!("Sector '{}' is prohibited", sector),
                        remediation: None,
                    });
                }
            }
        }
        
        ComplianceResult {
            compliant: violations.is_empty(),
            violations,
            strictness: self.strictness,
        }
    }
    
    /// Check if a sector is prohibited.
    fn is_prohibited_sector(&self, sector: &str) -> bool {
        let prohibited = [
            "alcohol", "tobacco", "gambling", "pork",
            "adult_entertainment", "weapons", "conventional_finance",
        ];
        
        let sector_lower = sector.to_lowercase();
        prohibited.iter().any(|p| sector_lower.contains(p))
    }
}

impl Default for ShariahCompliance {
    fn default() -> Self {
        Self::new()
    }
}

/// Compliance check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    /// Is the transaction compliant?
    pub compliant: bool,
    /// List of violations (if any)
    pub violations: Vec<ShariahViolation>,
    /// Strictness level used
    pub strictness: StrictnessLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_riba() {
        let checker = ShariahCompliance::new();
        let tx = TransactionCheck {
            transaction_type: TransactionType::Loan,
            amount: 10000,
            currency: "SAR".into(),
            sector: None,
            interest_rate: Some(0.05),
            uncertainty_level: None,
        };
        
        let result = checker.check_transaction(&tx);
        assert!(!result.compliant);
        assert!(result.violations.iter().any(|v| v.check == ShariahCheck::Riba));
    }

    #[test]
    fn test_detect_haram_sector() {
        let checker = ShariahCompliance::new();
        let tx = TransactionCheck {
            transaction_type: TransactionType::Investment,
            amount: 50000,
            currency: "MYR".into(),
            sector: Some("alcohol_production".into()),
            interest_rate: None,
            uncertainty_level: None,
        };
        
        let result = checker.check_transaction(&tx);
        assert!(!result.compliant);
        assert!(result.violations.iter().any(|v| v.check == ShariahCheck::HalalSector));
    }

    #[test]
    fn test_compliant_transaction() {
        let checker = ShariahCompliance::new();
        let tx = TransactionCheck {
            transaction_type: TransactionType::Trade,
            amount: 1000,
            currency: "AED".into(),
            sector: Some("technology".into()),
            interest_rate: None,
            uncertainty_level: Some(0.1),
        };
        
        let result = checker.check_transaction(&tx);
        assert!(result.compliant);
    }
}

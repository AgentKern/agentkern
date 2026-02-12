//! Cultural & Ethical Compliance Module
//!
//! Beyond finance - applies to ALL agent operations:
//!
//! | Domain | Islamic Example | Other Examples |
//! |--------|-----------------|----------------|
//! | **Content** | No haram imagery | COPPA for kids |
//! | **Food/Supply** | Halal sourcing | Kosher, Vegan, Organic |
//! | **Contracts** | No gharar | GDPR consent, ADA |
//! | **Time** | Prayer times | Sabbath, holidays |
//! | **Language** | Right-to-left | Locale, formality |

use serde::{Deserialize, Serialize};

/// Compliance framework types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceFramework {
    /// Islamic Shariah compliance
    Shariah,
    /// Jewish Halacha compliance
    Kosher,
    /// Hindu dietary/practice compliance
    Hindu,
    /// Catholic/Christian compliance
    Christian,
    /// Secular ethical (ESG, vegan, etc.)
    Secular,
    /// Child protection (COPPA, KOSA)
    ChildProtection,
    /// Accessibility (ADA, WCAG)
    Accessibility,
}

impl ComplianceFramework {
    /// Get regions where this framework is primary.
    pub fn primary_regions(&self) -> &[&str] {
        match self {
            Self::Shariah => &["SA", "AE", "MY", "ID", "PK", "TR", "QA", "BH", "KW"],
            Self::Kosher => &["IL"],
            Self::Hindu => &["IN", "NP"],
            Self::Christian => &["VA"],
            Self::Secular => &["*"], // Universal option
            Self::ChildProtection => &["US", "EU", "UK"],
            Self::Accessibility => &["US", "EU", "UK", "AU"],
        }
    }
}

/// Compliance check for any operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCheck {
    /// Operation type
    pub operation_type: OperationType,
    /// Context data
    pub context: OperationContext,
    /// Frameworks to check against
    pub frameworks: Vec<ComplianceFramework>,
}

/// Operation types that can be compliance-checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Content generation/display
    Content,
    /// Product sourcing/supply chain
    SupplyChain,
    /// Contract/agreement creation
    Contract,
    /// Scheduled task execution
    ScheduledTask,
    /// Communication/outreach
    Communication,
    /// Data processing
    DataProcessing,
    /// Financial transaction
    Financial,
}

/// Context for operation compliance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationContext {
    /// Target region
    pub region: Option<String>,
    /// Content categories involved
    pub content_categories: Vec<String>,
    /// Products/materials involved
    pub products: Vec<String>,
    /// Time of operation (for prayer time checks)
    pub timestamp: Option<i64>,
    /// Audience type
    pub audience: Option<AudienceType>,
}

/// Audience type for content compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudienceType {
    General,
    Children,
    Adults,
    Professional,
}

/// Result of compliance check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    /// Is the operation compliant?
    pub compliant: bool,
    /// Frameworks that passed
    pub passed: Vec<ComplianceFramework>,
    /// Frameworks that failed
    pub failed: Vec<FrameworkViolation>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Framework-specific violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkViolation {
    pub framework: ComplianceFramework,
    pub domain: String,
    pub reason: String,
    pub severity: ViolationSeverity,
}

/// Severity of violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Advisory only
    Advisory,
    /// Should fix
    Warning,
    /// Must fix
    Required,
    /// Legal requirement
    Legal,
}

/// Cultural compliance checker.
pub struct CulturalCompliance {
    frameworks: Vec<ComplianceFramework>,
}

impl CulturalCompliance {
    /// Create with specific frameworks.
    pub fn new(frameworks: Vec<ComplianceFramework>) -> Self {
        Self { frameworks }
    }
    
    /// Detect frameworks for a region.
    pub fn for_region(region: &str) -> Self {
        let mut frameworks = vec![ComplianceFramework::Secular];
        
        // Add region-specific frameworks
        let shariah_regions = ["SA", "AE", "MY", "ID", "PK", "TR", "QA", "BH", "KW", "EG", "JO", "OM"];
        if shariah_regions.contains(&region) {
            frameworks.push(ComplianceFramework::Shariah);
        }
        
        if region == "IL" {
            frameworks.push(ComplianceFramework::Kosher);
        }
        
        if region == "IN" || region == "NP" {
            frameworks.push(ComplianceFramework::Hindu);
        }
        
        // Child protection is global but stricter in some regions
        if ["US", "EU", "UK"].contains(&region) {
            frameworks.push(ComplianceFramework::ChildProtection);
        }
        
        Self { frameworks }
    }
    
    /// Check operation for compliance.
    pub fn check(&self, operation: &OperationCheck) -> ComplianceCheckResult {
        let mut passed = Vec::new();
        let mut failed = Vec::new();
        let mut recommendations = Vec::new();
        
        for framework in &self.frameworks {
            match self.check_framework(framework, operation) {
                Ok(recs) => {
                    passed.push(*framework);
                    recommendations.extend(recs);
                }
                Err(violation) => {
                    failed.push(violation);
                }
            }
        }
        
        ComplianceCheckResult {
            compliant: failed.is_empty(),
            passed,
            failed,
            recommendations,
        }
    }
    
    fn check_framework(
        &self,
        framework: &ComplianceFramework,
        operation: &OperationCheck,
    ) -> Result<Vec<String>, FrameworkViolation> {
        let mut recommendations = Vec::new();
        
        match framework {
            ComplianceFramework::Shariah => {
                // Check for haram content
                let haram_categories = ["alcohol", "pork", "gambling", "adult"];
                for cat in &operation.context.content_categories {
                    if haram_categories.iter().any(|h| cat.to_lowercase().contains(h)) {
                        return Err(FrameworkViolation {
                            framework: *framework,
                            domain: "Content".into(),
                            reason: format!("Category '{}' not permissible", cat),
                            severity: ViolationSeverity::Required,
                        });
                    }
                }
                
                // Check for halal products
                for product in &operation.context.products {
                    if product.to_lowercase().contains("pork") {
                        return Err(FrameworkViolation {
                            framework: *framework,
                            domain: "SupplyChain".into(),
                            reason: "Non-halal product detected".into(),
                            severity: ViolationSeverity::Required,
                        });
                    }
                }
                
                recommendations.push("Consider halal certification for supply chain".into());
            }
            
            ComplianceFramework::ChildProtection => {
                if operation.context.audience == Some(AudienceType::Children) {
                    // Stricter checks for child content
                    if operation.operation_type == OperationType::DataProcessing {
                        recommendations.push("COPPA: Obtain parental consent for data collection".into());
                    }
                }
            }
            
            ComplianceFramework::Accessibility => {
                if operation.operation_type == OperationType::Content {
                    recommendations.push("WCAG: Ensure alt text and screen reader compatibility".into());
                }
            }
            
            _ => {}
        }
        
        Ok(recommendations)
    }
}

impl Default for CulturalCompliance {
    fn default() -> Self {
        Self::new(vec![ComplianceFramework::Secular])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_framework_detection() {
        let saudi = CulturalCompliance::for_region("SA");
        assert!(saudi.frameworks.contains(&ComplianceFramework::Shariah));
        
        let us = CulturalCompliance::for_region("US");
        assert!(us.frameworks.contains(&ComplianceFramework::ChildProtection));
        assert!(!us.frameworks.contains(&ComplianceFramework::Shariah));
    }

    #[test]
    fn test_shariah_content_check() {
        let checker = CulturalCompliance::for_region("MY");
        let operation = OperationCheck {
            operation_type: OperationType::Content,
            context: OperationContext {
                content_categories: vec!["gambling".into()],
                ..Default::default()
            },
            frameworks: vec![ComplianceFramework::Shariah],
        };
        
        let result = checker.check(&operation);
        assert!(!result.compliant);
    }

    #[test]
    fn test_halal_supply_chain() {
        let checker = CulturalCompliance::for_region("ID");
        let operation = OperationCheck {
            operation_type: OperationType::SupplyChain,
            context: OperationContext {
                products: vec!["chicken".into(), "rice".into()],
                ..Default::default()
            },
            frameworks: vec![ComplianceFramework::Shariah],
        };
        
        let result = checker.check(&operation);
        assert!(result.compliant);
    }
}

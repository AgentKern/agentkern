//! Investment & Product Screening
//!
//! Multi-framework screening for:
//! - Halal investments (no haram sectors)
//! - Kosher products
//! - ESG scoring
//! - Ethical supply chain

use serde::{Deserialize, Serialize};

/// Screening criteria enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreeningCriteria {
    /// Islamic halal screening
    Halal,
    /// Jewish kosher screening
    Kosher,
    /// Hindu (no beef) screening
    Hindu,
    /// Vegan screening
    Vegan,
    /// Vegetarian screening
    Vegetarian,
    /// Organic screening
    Organic,
    /// Environmental (E)
    Environmental,
    /// Social (S)
    Social,
    /// Governance (G)
    Governance,
}

/// Screener for investments and products.
pub struct InvestmentScreener {
    criteria: Vec<ScreeningCriteria>,
}

impl InvestmentScreener {
    /// Create new screener with criteria.
    pub fn new(criteria: Vec<ScreeningCriteria>) -> Self {
        Self { criteria }
    }
    
    /// Halal-only screener.
    pub fn halal() -> Self {
        Self::new(vec![ScreeningCriteria::Halal])
    }
    
    /// Full ESG screener.
    pub fn esg() -> Self {
        Self::new(vec![
            ScreeningCriteria::Environmental,
            ScreeningCriteria::Social,
            ScreeningCriteria::Governance,
        ])
    }
    
    /// Screen an investment or product.
    pub fn screen(&self, target: &ScreenTarget) -> ScreeningResult {
        let mut passed = Vec::new();
        let mut failed = Vec::new();
        
        for criterion in &self.criteria {
            if self.check_criterion(criterion, target) {
                passed.push(*criterion);
            } else {
                failed.push(*criterion);
            }
        }
        
        ScreeningResult {
            approved: failed.is_empty(),
            passed,
            failed,
            score: (passed.len() as f64 / self.criteria.len() as f64) * 100.0,
        }
    }
    
    fn check_criterion(&self, criterion: &ScreeningCriteria, target: &ScreenTarget) -> bool {
        match criterion {
            ScreeningCriteria::Halal => {
                !target.sectors.iter().any(|s| {
                    let s_lower = s.to_lowercase();
                    ["alcohol", "gambling", "pork", "tobacco", "weapons", "adult"]
                        .iter()
                        .any(|h| s_lower.contains(h))
                })
            }
            ScreeningCriteria::Kosher => {
                !target.sectors.iter().any(|s| {
                    let s_lower = s.to_lowercase();
                    ["pork", "shellfish"].iter().any(|h| s_lower.contains(h))
                })
            }
            ScreeningCriteria::Hindu => {
                !target.sectors.iter().any(|s| s.to_lowercase().contains("beef"))
            }
            ScreeningCriteria::Vegan => {
                !target.sectors.iter().any(|s| {
                    let s_lower = s.to_lowercase();
                    ["meat", "dairy", "eggs", "leather", "wool"]
                        .iter()
                        .any(|h| s_lower.contains(h))
                })
            }
            ScreeningCriteria::Environmental => {
                target.esg_scores.as_ref().map(|s| s.environmental >= 50.0).unwrap_or(true)
            }
            ScreeningCriteria::Social => {
                target.esg_scores.as_ref().map(|s| s.social >= 50.0).unwrap_or(true)
            }
            ScreeningCriteria::Governance => {
                target.esg_scores.as_ref().map(|s| s.governance >= 50.0).unwrap_or(true)
            }
            _ => true,
        }
    }
}

/// Target to screen.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScreenTarget {
    /// Name/ID
    pub name: String,
    /// Sectors involved
    pub sectors: Vec<String>,
    /// ESG scores (if available)
    pub esg_scores: Option<EsgScores>,
}

/// ESG scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgScores {
    pub environmental: f64,
    pub social: f64,
    pub governance: f64,
}

/// Screening result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningResult {
    /// Overall approved?
    pub approved: bool,
    /// Criteria that passed
    pub passed: Vec<ScreeningCriteria>,
    /// Criteria that failed
    pub failed: Vec<ScreeningCriteria>,
    /// Score (0-100)
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_halal_screening_pass() {
        let screener = InvestmentScreener::halal();
        let target = ScreenTarget {
            name: "Tech Startup".into(),
            sectors: vec!["technology".into(), "software".into()],
            esg_scores: None,
        };
        
        let result = screener.screen(&target);
        assert!(result.approved);
    }

    #[test]
    fn test_halal_screening_fail() {
        let screener = InvestmentScreener::halal();
        let target = ScreenTarget {
            name: "Casino Corp".into(),
            sectors: vec!["gambling".into(), "entertainment".into()],
            esg_scores: None,
        };
        
        let result = screener.screen(&target);
        assert!(!result.approved);
    }

    #[test]
    fn test_multi_criteria_screening() {
        let screener = InvestmentScreener::new(vec![
            ScreeningCriteria::Halal,
            ScreeningCriteria::Environmental,
        ]);
        
        let target = ScreenTarget {
            name: "Green Energy".into(),
            sectors: vec!["renewable_energy".into()],
            esg_scores: Some(EsgScores {
                environmental: 85.0,
                social: 70.0,
                governance: 75.0,
            }),
        };
        
        let result = screener.screen(&target);
        assert!(result.approved);
        assert_eq!(result.score, 100.0);
    }
}

//! Trust Score Provider for Entra
//!
//! Provides VeriMantle trust scores to Microsoft Entra for Conditional Access

use serde::{Deserialize, Serialize};

/// Trust score provider.
pub struct TrustScoreProvider {
    /// Weight for each trust factor
    weights: TrustWeights,
}

/// Trust score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    /// Overall score (0.0 - 1.0)
    pub overall: f64,
    /// Individual factors
    pub factors: TrustFactors,
    /// Confidence in score
    pub confidence: f64,
    /// Calculated at
    pub calculated_at: String,
    /// Recommendation
    pub recommendation: TrustRecommendation,
}

/// Trust factors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustFactors {
    /// Identity verification score
    pub identity: f64,
    /// Behavioral consistency score
    pub behavior: f64,
    /// Policy compliance score
    pub compliance: f64,
    /// Historical reliability score
    pub reliability: f64,
    /// Security posture score
    pub security: f64,
}

/// Trust weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustWeights {
    pub identity: f64,
    pub behavior: f64,
    pub compliance: f64,
    pub reliability: f64,
    pub security: f64,
}

impl Default for TrustWeights {
    fn default() -> Self {
        Self {
            identity: 0.25,
            behavior: 0.20,
            compliance: 0.25,
            reliability: 0.15,
            security: 0.15,
        }
    }
}

/// Trust recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustRecommendation {
    /// Full access
    Allow,
    /// Require additional verification
    Challenge,
    /// Limit permissions
    Restrict,
    /// Block access
    Block,
}

impl TrustScoreProvider {
    /// Create new provider with default weights.
    pub fn new() -> Self {
        Self {
            weights: TrustWeights::default(),
        }
    }
    
    /// Create with custom weights.
    pub fn with_weights(weights: TrustWeights) -> Self {
        Self { weights }
    }
    
    /// Calculate trust score from factors.
    pub fn calculate(&self, factors: TrustFactors) -> TrustScore {
        let overall = 
            factors.identity * self.weights.identity +
            factors.behavior * self.weights.behavior +
            factors.compliance * self.weights.compliance +
            factors.reliability * self.weights.reliability +
            factors.security * self.weights.security;
        
        let recommendation = match overall {
            s if s >= 0.8 => TrustRecommendation::Allow,
            s if s >= 0.6 => TrustRecommendation::Challenge,
            s if s >= 0.4 => TrustRecommendation::Restrict,
            _ => TrustRecommendation::Block,
        };
        
        TrustScore {
            overall,
            factors,
            confidence: 0.9,
            calculated_at: chrono::Utc::now().to_rfc3339(),
            recommendation,
        }
    }
    
    /// Quick trust check.
    pub fn is_trusted(&self, factors: &TrustFactors, threshold: f64) -> bool {
        let score = self.calculate(factors.clone());
        score.overall >= threshold
    }
}

impl Default for TrustScoreProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_score_calculation() {
        let provider = TrustScoreProvider::new();
        
        let factors = TrustFactors {
            identity: 0.9,
            behavior: 0.8,
            compliance: 0.95,
            reliability: 0.85,
            security: 0.9,
        };
        
        let score = provider.calculate(factors);
        assert!(score.overall > 0.8);
        assert_eq!(score.recommendation, TrustRecommendation::Allow);
    }

    #[test]
    fn test_low_trust_score() {
        let provider = TrustScoreProvider::new();
        
        let factors = TrustFactors {
            identity: 0.3,
            behavior: 0.2,
            compliance: 0.4,
            reliability: 0.3,
            security: 0.2,
        };
        
        let score = provider.calculate(factors);
        assert!(score.overall < 0.4);
        assert_eq!(score.recommendation, TrustRecommendation::Block);
    }
}

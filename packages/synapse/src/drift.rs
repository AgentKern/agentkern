//! VeriMantle-Synapse: Drift Detection
//!
//! Detects when an agent has drifted from its original intent.
//!
//! Per ARCHITECTURE.md:
//! - Prevents "Intent Drift" by anchoring agents to business goals
//! - Uses semantic similarity when embeddings are available

use crate::intent::IntentPath;

/// Drift detection result.
#[derive(Debug, Clone)]
pub struct DriftResult {
    /// Has significant drift been detected?
    pub drifted: bool,
    /// Drift score (0-100)
    pub score: u8,
    /// Reason for drift detection
    pub reason: Option<String>,
}

/// Drift detector for intent paths.
pub struct DriftDetector {
    /// Score threshold for drift detection
    threshold: u8,
    /// Maximum allowed step overrun ratio
    max_overrun_ratio: f32,
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self {
            threshold: 50,
            max_overrun_ratio: 1.5,
        }
    }
}

impl DriftDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_threshold(mut self, threshold: u8) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_max_overrun(mut self, ratio: f32) -> Self {
        self.max_overrun_ratio = ratio;
        self
    }

    /// Check an intent path for drift.
    pub fn check(&self, path: &IntentPath) -> DriftResult {
        let mut score = 0u8;
        let mut reasons = Vec::new();

        // Check 1: Step overrun
        if path.expected_steps > 0 {
            let overrun_ratio = path.current_step as f32 / path.expected_steps as f32;
            if overrun_ratio > self.max_overrun_ratio {
                let overrun_score = ((overrun_ratio - 1.0) * 50.0).min(50.0) as u8;
                score = score.saturating_add(overrun_score);
                reasons.push(format!(
                    "Step overrun: {} steps taken, {} expected (ratio: {:.1}x)",
                    path.current_step, path.expected_steps, overrun_ratio
                ));
            }
        }

        // Check 2: Semantic similarity (if embeddings available)
        if let (Some(intent_emb), Some(last_step)) = (&path.intent_embedding, path.history.last()) {
            if let Some(step_emb) = &last_step.embedding {
                let similarity = cosine_similarity(intent_emb, step_emb);
                if similarity < 0.5 {
                    let semantic_score = ((1.0 - similarity) * 50.0) as u8;
                    score = score.saturating_add(semantic_score);
                    reasons.push(format!(
                        "Low semantic similarity: {:.2} (threshold: 0.5)",
                        similarity
                    ));
                }
            }
        }

        // Check 3: Action pattern anomaly
        // (Simple heuristic: repeated failures)
        let recent_failures = path.history.iter().rev().take(3)
            .filter(|s| s.result.as_ref().map(|r| r.contains("fail") || r.contains("error")).unwrap_or(false))
            .count();
        if recent_failures >= 2 {
            score = score.saturating_add(20);
            reasons.push(format!("{} recent failures detected", recent_failures));
        }

        let drifted = score >= self.threshold;
        let reason = if reasons.is_empty() {
            None
        } else {
            Some(reasons.join("; "))
        };

        DriftResult {
            drifted,
            score,
            reason,
        }
    }
}

/// Calculate cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    dot / (mag_a * mag_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift_on_normal_path() {
        let path = IntentPath::new("agent-1", "Test", 5);
        let detector = DriftDetector::new();
        
        let result = detector.check(&path);
        assert!(!result.drifted);
        assert_eq!(result.score, 0);
    }

    #[test]
    fn test_drift_on_overrun() {
        let mut path = IntentPath::new("agent-1", "Test", 2);
        path.record_step("step1", None);
        path.record_step("step2", None);
        path.record_step("step3", None);
        path.record_step("step4", None);  // 2x overrun
        
        let detector = DriftDetector::new().with_threshold(20);
        let result = detector.check(&path);
        
        assert!(result.drifted);
        assert!(result.score > 0);
        assert!(result.reason.unwrap().contains("overrun"));
    }

    #[test]
    fn test_drift_on_failures() {
        let mut path = IntentPath::new("agent-1", "Test", 10);
        path.record_step("step1", Some("failed".to_string()));
        path.record_step("step2", Some("error occurred".to_string()));
        path.record_step("step3", Some("failed again".to_string()));
        
        let detector = DriftDetector::new().with_threshold(15);
        let result = detector.check(&path);
        
        assert!(result.drifted);
        assert!(result.reason.unwrap().contains("failures"));
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }
}

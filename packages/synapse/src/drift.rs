//! AgentKern-Synapse: Drift Detection & Alerting
//!
//! Detects when an agent has drifted from its original intent
//! and sends alerts via webhooks or callbacks.
//!
//! Per ARCHITECTURE.md:
//! - Prevents "Intent Drift" by anchoring agents to business goals
//! - Uses semantic similarity when embeddings are available

use crate::intent::IntentPath;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

// ============================================================================
// DRIFT RESULT
// ============================================================================

/// Drift detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftResult {
    /// Has significant drift been detected?
    pub drifted: bool,
    /// Drift score (0-100)
    pub score: u8,
    /// Reason for drift detection
    pub reason: Option<String>,
}

// ============================================================================
// DRIFT ALERTING
// ============================================================================

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational - slight drift
    Info,
    /// Warning - moderate drift
    Warning,
    /// Critical - severe drift, may need intervention
    Critical,
}

impl AlertSeverity {
    /// Determine severity from drift score.
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=40 => AlertSeverity::Info,
            41..=70 => AlertSeverity::Warning,
            _ => AlertSeverity::Critical,
        }
    }
}

/// Drift alert for notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAlert {
    /// Alert ID
    pub id: String,
    /// Agent ID
    pub agent_id: String,
    /// Intent path ID
    pub path_id: String,
    /// Original intent
    pub original_intent: String,
    /// Drift result
    pub drift_result: DriftResult,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Current step
    pub current_step: u32,
    /// Expected steps
    pub expected_steps: u32,
}

impl DriftAlert {
    /// Create a new drift alert.
    pub fn new(path: &IntentPath, result: DriftResult) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: path.agent_id.clone(),
            path_id: path.id.to_string(),
            original_intent: path.original_intent.clone(),
            drift_result: result.clone(),
            severity: AlertSeverity::from_score(result.score),
            timestamp: Utc::now(),
            current_step: path.current_step,
            expected_steps: path.expected_steps,
        }
    }
}

/// Webhook configuration.
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// Webhook URL
    pub url: String,
    /// Minimum severity to trigger
    pub min_severity: AlertSeverity,
    /// Headers to include
    pub headers: Vec<(String, String)>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

impl WebhookConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            min_severity: AlertSeverity::Warning,
            headers: vec![],
            timeout_ms: 5000,
        }
    }

    pub fn with_min_severity(mut self, severity: AlertSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}

/// Callback function type for drift alerts.
pub type AlertCallback = Box<dyn Fn(&DriftAlert) + Send + Sync>;

/// Drift alerter for sending notifications.
pub struct DriftAlerter {
    /// Registered webhooks
    webhooks: Arc<RwLock<Vec<WebhookConfig>>>,
    /// Registered callbacks
    callbacks: Arc<RwLock<Vec<AlertCallback>>>,
    /// Alert history
    history: Arc<RwLock<Vec<DriftAlert>>>,
    /// Maximum history size
    max_history: usize,
}

impl Default for DriftAlerter {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftAlerter {
    /// Create a new drift alerter.
    pub fn new() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(Vec::new())),
            callbacks: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 1000,
        }
    }

    /// Register a webhook for alerts.
    pub fn register_webhook(&self, config: WebhookConfig) {
        let mut webhooks = self.webhooks.write();
        webhooks.push(config);
    }

    /// Register a callback for alerts.
    pub fn on_alert(&self, callback: AlertCallback) {
        let mut callbacks = self.callbacks.write();
        callbacks.push(callback);
    }

    /// Send an alert.
    pub async fn send_alert(&self, alert: DriftAlert) {
        // Store in history
        {
            let mut history = self.history.write();
            history.push(alert.clone());
            if history.len() > self.max_history {
                history.remove(0);
            }
        }

        // Invoke callbacks
        {
            let callbacks = self.callbacks.read();
            for callback in callbacks.iter() {
                callback(&alert);
            }
        }

        // Send to webhooks
        let webhooks = self.webhooks.read().clone();
        for webhook in webhooks {
            if alert.severity >= webhook.min_severity {
                self.send_to_webhook(&webhook, &alert).await;
            }
        }
    }

    /// Send alert to a webhook.
    /// Graceful fallback: tries real HTTP, falls back to logging on failure.
    async fn send_to_webhook(&self, config: &WebhookConfig, alert: &DriftAlert) {
        // Try actual HTTP POST
        let client = reqwest::Client::new();
        
        let mut request = client
            .post(&config.url)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_millis(config.timeout_ms));
        
        // Add custom headers
        for (key, value) in &config.headers {
            request = request.header(key.as_str(), value.as_str());
        }
        
        // Send the request
        match request.json(alert).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::info!(
                        webhook_url = %config.url,
                        alert_id = %alert.id,
                        "Drift alert sent successfully"
                    );
                } else {
                    tracing::warn!(
                        webhook_url = %config.url,
                        status = %response.status(),
                        "Webhook returned non-success status"
                    );
                }
            }
            Err(e) => {
                // Graceful fallback: log the alert that would have been sent
                tracing::warn!(
                    webhook_url = %config.url,
                    alert_id = %alert.id,
                    agent_id = %alert.agent_id,
                    severity = ?alert.severity,
                    error = %e,
                    "Webhook failed, alert logged locally"
                );
            }
        }
    }


    /// Get recent alerts.
    pub fn get_history(&self, limit: usize) -> Vec<DriftAlert> {
        let history = self.history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get alerts for a specific agent.
    pub fn get_alerts_for_agent(&self, agent_id: &str) -> Vec<DriftAlert> {
        let history = self.history.read();
        history.iter()
            .filter(|a| a.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Get alert count by severity.
    pub fn get_alert_counts(&self) -> (usize, usize, usize) {
        let history = self.history.read();
        let info = history.iter().filter(|a| a.severity == AlertSeverity::Info).count();
        let warning = history.iter().filter(|a| a.severity == AlertSeverity::Warning).count();
        let critical = history.iter().filter(|a| a.severity == AlertSeverity::Critical).count();
        (info, warning, critical)
    }
}

// ============================================================================
// DRIFT DETECTOR
// ============================================================================

/// Drift detector for intent paths.
pub struct DriftDetector {
    /// Score threshold for drift detection
    threshold: u8,
    /// Maximum allowed step overrun ratio
    max_overrun_ratio: f32,
    /// Optional alerter
    alerter: Option<Arc<DriftAlerter>>,
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self {
            threshold: 50,
            max_overrun_ratio: 1.5,
            alerter: None,
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

    /// Attach an alerter for automatic notifications.
    pub fn with_alerter(mut self, alerter: Arc<DriftAlerter>) -> Self {
        self.alerter = Some(alerter);
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

        // Check 3: Action pattern anomaly (repeated failures)
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

    /// Check for drift and automatically alert if detected.
    pub async fn check_and_alert(&self, path: &IntentPath) -> DriftResult {
        let result = self.check(path);

        if result.drifted {
            if let Some(ref alerter) = self.alerter {
                let alert = DriftAlert::new(path, result.clone());
                alerter.send_alert(alert).await;
            }
        }

        result
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

    #[test]
    fn test_alert_severity() {
        assert_eq!(AlertSeverity::from_score(30), AlertSeverity::Info);
        assert_eq!(AlertSeverity::from_score(50), AlertSeverity::Warning);
        assert_eq!(AlertSeverity::from_score(80), AlertSeverity::Critical);
    }

    #[test]
    fn test_drift_alert_creation() {
        let path = IntentPath::new("agent-1", "Process order", 5);
        let result = DriftResult {
            drifted: true,
            score: 60,
            reason: Some("Step overrun".to_string()),
        };
        
        let alert = DriftAlert::new(&path, result);
        
        assert_eq!(alert.agent_id, "agent-1");
        assert_eq!(alert.severity, AlertSeverity::Warning);
    }

    #[test]
    fn test_alerter_history() {
        let alerter = DriftAlerter::new();
        let path = IntentPath::new("agent-1", "Test", 5);
        let result = DriftResult {
            drifted: true,
            score: 75,
            reason: Some("Test".to_string()),
        };
        
        let alert = DriftAlert::new(&path, result);
        
        // Use tokio runtime for async
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            alerter.send_alert(alert).await;
        });
        
        let history = alerter.get_history(10);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_alerter_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let alerter = DriftAlerter::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();
        
        alerter.on_alert(Box::new(move |_| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        }));
        
        let path = IntentPath::new("agent-1", "Test", 5);
        let result = DriftResult {
            drifted: true,
            score: 80,
            reason: None,
        };
        let alert = DriftAlert::new(&path, result);
        
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            alerter.send_alert(alert).await;
        });
        
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
}


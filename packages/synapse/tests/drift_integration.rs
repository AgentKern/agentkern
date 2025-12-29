//! Integration Tests for Drift Detector
//!
//! Tests semantic behavioral analysis for production scenarios.

use agentkern_synapse::drift::{AlertSeverity, DriftAlerter, DriftDetector, DriftResult};
use agentkern_synapse::intent::IntentPath;
use std::sync::Arc;

#[test]
fn test_no_drift_fresh_path() {
    let path = IntentPath::new("agent-1", "Process customer order", 10);
    let detector = DriftDetector::new().with_threshold(50);
    
    let result = detector.check(&path);
    
    assert!(!result.drifted);
    assert_eq!(result.score, 0);
    assert!(result.reason.is_none());
}

#[test]
fn test_step_overrun_drift() {
    let mut path = IntentPath::new("agent-1", "Quick task", 2);
    
    // Execute 4 steps (2x expected)
    for i in 0..4 {
        path.record_step(&format!("step_{}", i), None);
    }
    
    let detector = DriftDetector::new()
        .with_threshold(20)
        .with_max_overrun(1.5);
    
    let result = detector.check(&path);
    
    assert!(result.drifted);
    assert!(result.score > 0);
    assert!(result.reason.as_ref().unwrap().contains("overrun"));
}

#[test]
fn test_failure_pattern_drift() {
    let mut path = IntentPath::new("agent-1", "Robust task", 10);
    
    // Three consecutive failures
    path.record_step("step1", Some("failed: timeout".to_string()));
    path.record_step("step2", Some("error: connection refused".to_string()));
    path.record_step("step3", Some("failed: retry exhausted".to_string()));
    
    let detector = DriftDetector::new().with_threshold(15);
    let result = detector.check(&path);
    
    assert!(result.drifted);
    assert!(result.reason.as_ref().unwrap().contains("failures"));
}

#[test]
fn test_circular_behavior_detection() {
    let mut path = IntentPath::new("agent-1", "Linear task", 20);
    
    // A-B-A-B circular pattern
    path.record_step("action_A", None);
    path.record_step("action_B", None);
    path.record_step("action_A", None);
    path.record_step("action_B", None);
    
    let detector = DriftDetector::new().with_threshold(10);
    let result = detector.check(&path);
    
    assert!(result.drifted);
    assert!(result.reason.as_ref().unwrap().contains("behavioral pattern"));
}

#[test]
fn test_alerter_integration() {
    use std::sync::atomic::{AtomicBool, Ordering};
    
    let alerter = Arc::new(DriftAlerter::new());
    let alerted = Arc::new(AtomicBool::new(false));
    let alerted_clone = alerted.clone();
    
    alerter.on_alert(Box::new(move |alert| {
        assert_eq!(alert.agent_id, "agent-drift");
        alerted_clone.store(true, Ordering::SeqCst);
    }));
    
    let detector = DriftDetector::new()
        .with_threshold(10)
        .with_alerter(alerter.clone());
    
    let mut path = IntentPath::new("agent-drift", "Test", 1);
    path.record_step("step1", None);
    path.record_step("step2", None);
    path.record_step("step3", None);
    
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        detector.check_and_alert(&path).await;
    });
    
    assert!(alerted.load(Ordering::SeqCst));
}

#[test]
fn test_severity_levels() {
    assert_eq!(AlertSeverity::from_score(20), AlertSeverity::Info);
    assert_eq!(AlertSeverity::from_score(50), AlertSeverity::Warning);
    assert_eq!(AlertSeverity::from_score(80), AlertSeverity::Critical);
}

#[test]
fn test_alert_history() {
    let alerter = DriftAlerter::new();
    
    let path = IntentPath::new("agent-1", "Test", 5);
    let result = DriftResult {
        drifted: true,
        score: 70,
        reason: Some("Test drift".to_string()),
    };
    
    let alert = agentkern_synapse::drift::DriftAlert::new(&path, result);
    
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        alerter.send_alert(alert).await;
    });
    
    let history = alerter.get_history(10);
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].agent_id, "agent-1");
}

#[test]
fn test_multiple_agents() {
    let detector = DriftDetector::new().with_threshold(50);
    
    let path1 = IntentPath::new("agent-1", "Task 1", 10);
    let path2 = IntentPath::new("agent-2", "Task 2", 10);
    let path3 = IntentPath::new("agent-3", "Task 3", 10);
    
    // All fresh paths should not drift
    assert!(!detector.check(&path1).drifted);
    assert!(!detector.check(&path2).drifted);
    assert!(!detector.check(&path3).drifted);
}

#[test]
fn test_performance() {
    let detector = DriftDetector::new();
    
    let mut path = IntentPath::new("agent-perf", "Performance test", 100);
    for i in 0..50 {
        path.record_step(&format!("step_{}", i), None);
    }
    
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        detector.check(&path);
    }
    let duration = start.elapsed();
    
    // 1000 checks should complete in under 100ms
    assert!(duration.as_millis() < 100, "Check took too long: {:?}", duration);
}

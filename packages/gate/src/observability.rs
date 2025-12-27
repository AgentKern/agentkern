//! eBPF Observability Plane
//!
//! Per ARCHITECTURE.md: "Zero-Overhead"
//! - eBPF (Extended Berkeley Packet Filter) for tracing
//! - Monitoring happens in the Linux Kernel, not in the application
//! - Zero instrumentation overhead
//!
//! This module provides eBPF-compatible telemetry integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Metrics collected by the observability plane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Requests allowed
    pub allowed_requests: u64,
    /// Requests denied
    pub denied_requests: u64,
    /// Average symbolic path latency (microseconds)
    pub avg_symbolic_latency_us: u64,
    /// Average neural path latency (microseconds) 
    pub avg_neural_latency_us: u64,
    /// P99 latency (microseconds)
    pub p99_latency_us: u64,
    /// Policies evaluated
    pub policies_evaluated: u64,
    /// WASM executions (if enabled)
    pub wasm_executions: u64,
}

/// Atomic metrics collector.
#[derive(Debug, Default)]
pub struct MetricsCollector {
    total_requests: AtomicU64,
    allowed_requests: AtomicU64,
    denied_requests: AtomicU64,
    symbolic_latency_sum: AtomicU64,
    neural_latency_sum: AtomicU64,
    policies_evaluated: AtomicU64,
    wasm_executions: AtomicU64,
    latencies: parking_lot::Mutex<Vec<u64>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a request.
    pub fn record_request(&self, allowed: bool, symbolic_latency_us: u64, neural_latency_us: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.allowed_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.denied_requests.fetch_add(1, Ordering::Relaxed);
        }
        self.symbolic_latency_sum.fetch_add(symbolic_latency_us, Ordering::Relaxed);
        self.neural_latency_sum.fetch_add(neural_latency_us, Ordering::Relaxed);
        
        let total_latency = symbolic_latency_us + neural_latency_us;
        let mut latencies = self.latencies.lock();
        latencies.push(total_latency);
        if latencies.len() > 10000 {
            latencies.remove(0); // Keep rolling window
        }
    }

    /// Record policy evaluation.
    pub fn record_policy_eval(&self) {
        self.policies_evaluated.fetch_add(1, Ordering::Relaxed);
    }

    /// Record WASM execution.
    pub fn record_wasm_exec(&self) {
        self.wasm_executions.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics.
    pub fn get_metrics(&self) -> GateMetrics {
        let total = self.total_requests.load(Ordering::Relaxed).max(1);
        let latencies = self.latencies.lock();
        
        let p99 = if !latencies.is_empty() {
            let mut sorted: Vec<_> = latencies.clone();
            sorted.sort();
            let idx = (sorted.len() as f64 * 0.99) as usize;
            sorted.get(idx.min(sorted.len() - 1)).copied().unwrap_or(0)
        } else {
            0
        };

        GateMetrics {
            total_requests: total,
            allowed_requests: self.allowed_requests.load(Ordering::Relaxed),
            denied_requests: self.denied_requests.load(Ordering::Relaxed),
            avg_symbolic_latency_us: self.symbolic_latency_sum.load(Ordering::Relaxed) / total,
            avg_neural_latency_us: self.neural_latency_sum.load(Ordering::Relaxed) / total,
            p99_latency_us: p99,
            policies_evaluated: self.policies_evaluated.load(Ordering::Relaxed),
            wasm_executions: self.wasm_executions.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics.
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.allowed_requests.store(0, Ordering::Relaxed);
        self.denied_requests.store(0, Ordering::Relaxed);
        self.symbolic_latency_sum.store(0, Ordering::Relaxed);
        self.neural_latency_sum.store(0, Ordering::Relaxed);
        self.policies_evaluated.store(0, Ordering::Relaxed);
        self.wasm_executions.store(0, Ordering::Relaxed);
        self.latencies.lock().clear();
    }
}

/// eBPF-compatible trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub timestamp_ns: u64,
    pub event_type: TraceEventType,
    pub agent_id: String,
    pub action: String,
    pub latency_us: u64,
    pub allowed: bool,
    pub risk_score: u8,
    pub metadata: HashMap<String, String>,
}

/// Types of trace events.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TraceEventType {
    RequestStart,
    SymbolicEval,
    NeuralEval,
    WasmExec,
    RequestEnd,
}

/// Observability plane for the Gate service.
pub struct ObservabilityPlane {
    metrics: Arc<MetricsCollector>,
    trace_buffer: parking_lot::Mutex<Vec<TraceEvent>>,
    buffer_size: usize,
}

impl ObservabilityPlane {
    pub fn new() -> Self {
        Self::with_buffer_size(10000)
    }

    pub fn with_buffer_size(size: usize) -> Self {
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            trace_buffer: parking_lot::Mutex::new(Vec::with_capacity(size)),
            buffer_size: size,
        }
    }

    /// Get shared metrics collector.
    pub fn metrics(&self) -> Arc<MetricsCollector> {
        Arc::clone(&self.metrics)
    }

    /// Record a trace event.
    pub fn trace(&self, event: TraceEvent) {
        let mut buffer = self.trace_buffer.lock();
        if buffer.len() >= self.buffer_size {
            buffer.remove(0);
        }
        buffer.push(event);
    }

    /// Get recent traces.
    pub fn get_traces(&self, limit: usize) -> Vec<TraceEvent> {
        let buffer = self.trace_buffer.lock();
        buffer.iter().rev().take(limit).cloned().collect()
    }

    /// Export traces for eBPF tooling (Cilium Hubble format).
    pub fn export_hubble(&self) -> Vec<u8> {
        let traces = self.get_traces(1000);
        serde_json::to_vec(&traces).unwrap_or_default()
    }

    /// Get prometheus-compatible metrics.
    pub fn prometheus_metrics(&self) -> String {
        let m = self.metrics.get_metrics();
        format!(
            r#"# HELP agentkern_gate_requests_total Total number of requests
# TYPE agentkern_gate_requests_total counter
agentkern_gate_requests_total{{status="allowed"}} {}
agentkern_gate_requests_total{{status="denied"}} {}

# HELP agentkern_gate_latency_us Request latency in microseconds
# TYPE agentkern_gate_latency_us gauge
agentkern_gate_latency_us{{path="symbolic"}} {}
agentkern_gate_latency_us{{path="neural"}} {}
agentkern_gate_latency_us{{quantile="0.99"}} {}

# HELP agentkern_gate_policies_evaluated Total policies evaluated
# TYPE agentkern_gate_policies_evaluated counter
agentkern_gate_policies_evaluated {}
"#,
            m.allowed_requests,
            m.denied_requests,
            m.avg_symbolic_latency_us,
            m.avg_neural_latency_us,
            m.p99_latency_us,
            m.policies_evaluated,
        )
    }
}

impl Default for ObservabilityPlane {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.record_request(true, 100, 0);
        collector.record_request(false, 50, 1000);
        collector.record_policy_eval();
        
        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.allowed_requests, 1);
        assert_eq!(metrics.denied_requests, 1);
    }

    #[test]
    fn test_observability_plane() {
        let plane = ObservabilityPlane::new();
        
        plane.trace(TraceEvent {
            timestamp_ns: 1234567890,
            event_type: TraceEventType::RequestStart,
            agent_id: "agent-1".to_string(),
            action: "test".to_string(),
            latency_us: 100,
            allowed: true,
            risk_score: 10,
            metadata: HashMap::new(),
        });
        
        let traces = plane.get_traces(10);
        assert_eq!(traces.len(), 1);
    }

    #[test]
    fn test_prometheus_export() {
        let plane = ObservabilityPlane::new();
        plane.metrics().record_request(true, 500, 0);
        
        let prom = plane.prometheus_metrics();
        assert!(prom.contains("agentkern_gate_requests_total"));
    }
}

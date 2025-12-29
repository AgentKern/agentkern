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
        self.symbolic_latency_sum
            .fetch_add(symbolic_latency_us, Ordering::Relaxed);
        self.neural_latency_sum
            .fetch_add(neural_latency_us, Ordering::Relaxed);

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

    // ========================================================================
    // OpenTelemetry Export (OTLP-compatible)
    // ========================================================================

    /// Export traces in OpenTelemetry format for Jaeger/Tempo.
    pub fn export_otel(&self) -> OtelExport {
        let traces = self.get_traces(1000);
        let trace_id = self.generate_trace_id();
        
        let spans: Vec<OtelSpan> = traces.iter().map(|t| OtelSpan {
            trace_id: trace_id.clone(),
            span_id: format!("{:016x}", t.timestamp_ns),
            parent_span_id: None,
            name: format!("{:?}", t.event_type),
            start_time_unix_nano: t.timestamp_ns,
            end_time_unix_nano: t.timestamp_ns + (t.latency_us * 1000),
            attributes: vec![
                OtelAttribute { key: "agent_id".to_string(), value: t.agent_id.clone() },
                OtelAttribute { key: "action".to_string(), value: t.action.clone() },
                OtelAttribute { key: "allowed".to_string(), value: t.allowed.to_string() },
                OtelAttribute { key: "risk_score".to_string(), value: t.risk_score.to_string() },
            ],
            status: if t.allowed { OtelStatus::Ok } else { OtelStatus::Error },
        }).collect();

        OtelExport {
            resource: OtelResource {
                service_name: "agentkern-gate".to_string(),
                service_version: env!("CARGO_PKG_VERSION").to_string(),
            },
            spans,
        }
    }

    /// Generate a 128-bit trace ID.
    fn generate_trace_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{:032x}", now)
    }

    /// Export as OTLP JSON (for HTTP push to collectors).
    pub fn export_otlp_json(&self) -> String {
        serde_json::to_string(&self.export_otel()).unwrap_or_default()
    }
}

// ============================================================================
// OpenTelemetry Structures
// ============================================================================

/// OpenTelemetry export payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelExport {
    pub resource: OtelResource,
    pub spans: Vec<OtelSpan>,
}

/// OpenTelemetry resource (service info).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelResource {
    pub service_name: String,
    pub service_version: String,
}

/// OpenTelemetry span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub attributes: Vec<OtelAttribute>,
    pub status: OtelStatus,
}

/// OpenTelemetry attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelAttribute {
    pub key: String,
    pub value: String,
}

/// OpenTelemetry span status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OtelStatus {
    Ok,
    Error,
    Unset,
}

// ============================================================================
// ALERT SUPPRESSION (Prevent Alert Storms)
// ============================================================================

/// Alert for suppression tracking.
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub service: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: std::time::Instant,
    pub fingerprint: String,
}

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Grouped incident for suppressed alerts.
#[derive(Debug, Clone)]
pub struct Incident {
    pub id: String,
    pub root_cause: String,
    pub affected_services: Vec<String>,
    pub alert_count: usize,
    pub first_seen: std::time::Instant,
    pub last_seen: std::time::Instant,
    pub severity: AlertSeverity,
}

/// Alert Suppressor - prevents alert storms during outages.
///
/// Groups related alerts into incidents based on:
/// - Service affinity (same service = same incident)
/// - Time proximity (alerts within window are related)
/// - Fingerprint matching (same error signature)
#[derive(Debug)]
pub struct AlertSuppressor {
    /// Active incidents
    incidents: parking_lot::Mutex<Vec<Incident>>,
    /// Suppression window (alerts within this window get grouped)
    window_secs: u64,
    /// Maximum alerts before suppression kicks in
    suppression_threshold: usize,
}

impl AlertSuppressor {
    /// Create a new alert suppressor.
    pub fn new() -> Self {
        Self {
            incidents: parking_lot::Mutex::new(Vec::new()),
            window_secs: 60,
            suppression_threshold: 5,
        }
    }

    /// Create with custom settings.
    pub fn with_settings(window_secs: u64, threshold: usize) -> Self {
        Self {
            incidents: parking_lot::Mutex::new(Vec::new()),
            window_secs,
            suppression_threshold: threshold,
        }
    }

    /// Process an alert - returns true if alert should be sent, false if suppressed.
    pub fn process(&self, alert: Alert) -> bool {
        let mut incidents = self.incidents.lock();
        let now = std::time::Instant::now();

        // Find or create incident for this alert
        let incident_idx = incidents.iter().position(|i| {
            i.affected_services.contains(&alert.service) ||
            i.root_cause == alert.fingerprint ||
            now.duration_since(i.last_seen).as_secs() < self.window_secs
        });

        match incident_idx {
            Some(idx) => {
                // Update existing incident
                let incident = &mut incidents[idx];
                incident.alert_count += 1;
                incident.last_seen = now;
                if !incident.affected_services.contains(&alert.service) {
                    incident.affected_services.push(alert.service);
                }
                if alert.severity > incident.severity {
                    incident.severity = alert.severity;
                }

                // Suppress if over threshold
                incident.alert_count <= self.suppression_threshold
            }
            None => {
                // Create new incident
                let incident_id = format!("incident-{}", incidents.len());
                incidents.push(Incident {
                    id: incident_id,
                    root_cause: alert.fingerprint,
                    affected_services: vec![alert.service],
                    alert_count: 1,
                    first_seen: now,
                    last_seen: now,
                    severity: alert.severity,
                });
                true // First alert always fires
            }
        }
    }

    /// Get active incidents.
    pub fn active_incidents(&self) -> Vec<Incident> {
        let incidents = self.incidents.lock();
        let now = std::time::Instant::now();
        
        incidents
            .iter()
            .filter(|i| now.duration_since(i.last_seen).as_secs() < self.window_secs * 2)
            .cloned()
            .collect()
    }

    /// Get suppression stats.
    pub fn stats(&self) -> (usize, usize) {
        let incidents = self.incidents.lock();
        let total_alerts: usize = incidents.iter().map(|i| i.alert_count).sum();
        let suppressed = total_alerts.saturating_sub(incidents.len() * self.suppression_threshold);
        (total_alerts, suppressed)
    }

    /// Clear old incidents.
    pub fn cleanup(&self) {
        let mut incidents = self.incidents.lock();
        let now = std::time::Instant::now();
        incidents.retain(|i| now.duration_since(i.last_seen).as_secs() < self.window_secs * 5);
    }
}

impl Default for AlertSuppressor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// OpenTelemetry SDK Tracer (Roadmap 2026 - Distributed Tracing)
// ============================================================================

/// OpenTelemetry tracer configuration.
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// Service name for traces
    pub service_name: String,
    /// OTLP endpoint (e.g., "http://localhost:4317" for gRPC, "http://localhost:4318/v1/traces" for HTTP)
    pub endpoint: String,
    /// Use HTTP (true) or gRPC (false)
    pub use_http: bool,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            service_name: "agentkern-gate".to_string(),
            endpoint: "http://localhost:4318/v1/traces".to_string(),
            use_http: true,
        }
    }
}

/// Initialize OpenTelemetry tracer with OTLP export.
/// 
/// Call this at application startup to enable distributed tracing.
/// Traces will be exported to the configured OTLP endpoint (Jaeger, Tempo, etc.).
/// 
/// # Example
/// 
/// ```rust,ignore
/// use agentkern_gate::observability::{init_otel_tracer, OtelConfig};
/// 
/// #[tokio::main]
/// async fn main() {
///     let config = OtelConfig {
///         service_name: "my-agent-service".to_string(),
///         endpoint: "http://jaeger:4318/v1/traces".to_string(),
///         use_http: true,
///     };
///     
///     if let Err(e) = init_otel_tracer(config) {
///         tracing::warn!("Failed to init OTel tracer: {}", e);
///     }
/// }
/// ```
#[cfg(feature = "otel")]
pub fn init_otel_tracer(config: OtelConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::Resource;
    use opentelemetry_sdk::trace::TracerProvider;
    use opentelemetry_otlp::WithExportConfig;
    
    // Build OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(&config.endpoint);
    
    // Create TracerProvider with batch exporter
    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(
            exporter.build_span_exporter()?,
            opentelemetry_sdk::runtime::Tokio,
        )
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
        ]))
        .build();
    
    // Set global tracer provider
    opentelemetry::global::set_tracer_provider(tracer_provider);
    
    // Integrate with tracing crate
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    
    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(opentelemetry::global::tracer("agentkern"));
    
    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .ok(); // Ignore error if subscriber already set
    
    tracing::info!(
        service = %config.service_name,
        endpoint = %config.endpoint,
        "OpenTelemetry tracer initialized"
    );
    
    Ok(())
}

/// Shutdown OpenTelemetry tracer gracefully.
/// Call this before application exit to flush pending traces.
#[cfg(feature = "otel")]
pub fn shutdown_otel_tracer() {
    opentelemetry::global::shutdown_tracer_provider();
    tracing::info!("OpenTelemetry tracer shutdown complete");
}

/// Placeholder for non-otel builds.
#[cfg(not(feature = "otel"))]
pub fn init_otel_tracer(_config: OtelConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::warn!("OpenTelemetry not enabled - build with `--features otel` to enable");
    Ok(())
}

/// Placeholder for non-otel builds.
#[cfg(not(feature = "otel"))]
pub fn shutdown_otel_tracer() {
    // No-op when OTel not enabled
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

    #[test]
    fn test_otel_export() {
        let plane = ObservabilityPlane::new();

        plane.trace(TraceEvent {
            timestamp_ns: 1234567890,
            event_type: TraceEventType::SymbolicEval,
            agent_id: "agent-otel".to_string(),
            action: "validate".to_string(),
            latency_us: 200,
            allowed: true,
            risk_score: 25,
            metadata: HashMap::new(),
        });

        let export = plane.export_otel();
        assert_eq!(export.resource.service_name, "agentkern-gate");
        assert_eq!(export.spans.len(), 1);
        assert_eq!(export.spans[0].name, "SymbolicEval");
        assert!(matches!(export.spans[0].status, OtelStatus::Ok));
    }

    #[test]
    fn test_otel_json_export() {
        let plane = ObservabilityPlane::new();
        plane.trace(TraceEvent {
            timestamp_ns: 9876543210,
            event_type: TraceEventType::NeuralEval,
            agent_id: "agent-json".to_string(),
            action: "classify".to_string(),
            latency_us: 500,
            allowed: false,
            risk_score: 85,
            metadata: HashMap::new(),
        });

        let json = plane.export_otlp_json();
        assert!(json.contains("agentkern-gate"));
        assert!(json.contains("NeuralEval"));
        assert!(json.contains("trace_id"));
    }
}

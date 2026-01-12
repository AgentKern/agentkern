//! AgentKern-Pulse: Unified Observability & Health reporting
//!
//! Provides a standardized way for all pillars to report semantic health,
//! carbon-aware metrics, and performance counters.

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use prometheus::{
    Counter, Gauge, Registry, opts, register_counter_with_registry, register_gauge_with_registry,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

lazy_static! {
    /// Global registry for AgentKern metrics
    pub static ref REGISTRY: Registry = Registry::new();

    /// Global carbon intensity gauge
    pub static ref CARBON_INTENSITY: Gauge = register_gauge_with_registry!(
        opts!("agentkern_carbon_intensity_g_kwh", "Current grid carbon intensity in gCO2eq/kWh"),
        REGISTRY
    ).unwrap();

    /// Global transaction counter
    pub static ref TX_COUNTER: Counter = register_counter_with_registry!(
        opts!("agentkern_transactions_total", "Total number of agentic transactions"),
        REGISTRY
    ).unwrap();
}

/// Health status of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Everything is normal
    Healthy,
    /// System is operational but stressed or degraded (e.g. high latency, high carbon)
    Degraded,
    /// System is failing or circuit broken
    Critical,
}

/// Semantic health report including business and sustainability metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticHealthReport {
    pub component: String,
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    /// gCO2eq/kWh
    pub carbon_intensity: f64,
    /// Normalized cost (e.g. 0.0 to 1.0)
    pub cost_index: f64,
    pub latency_ms: u64,
    pub uptime_secs: u64,
    pub message: String,
}

/// Trait for components that can report their pulse.
#[async_trait::async_trait]
pub trait Pulse {
    async fn get_health(&self) -> SemanticHealthReport;
}

pub struct PulseManager {
    registry: Arc<Registry>,
}

impl PulseManager {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(REGISTRY.clone()),
        }
    }

    pub fn get_registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }

    /// Report carbon intensity to the global gauge.
    pub fn report_carbon(&self, intensity: f64) {
        CARBON_INTENSITY.set(intensity);
    }

    /// Increment a global transaction hit.
    pub fn inc_tx(&self) {
        TX_COUNTER.inc();
    }
}

impl Default for PulseManager {
    fn default() -> Self {
        Self::new()
    }
}

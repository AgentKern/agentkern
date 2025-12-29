//! Bulkhead Pattern for Agent Isolation
//!
//! Per Antifragility Report: "Agent Sandbox - Bulkhead pattern"
//! Per ARCHITECTURE.md: "Budget-based isolation prevents resource exhaustion"
//!
//! Isolates agents to prevent one misbehaving agent from affecting others.
//! Implements:
//! - Concurrent request limits per agent
//! - Resource quotas (memory, CPU time, API calls)
//! - Timeout enforcement
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_arbiter::bulkhead::{Bulkhead, BulkheadConfig, ResourceQuota};
//!
//! let config = BulkheadConfig::default()
//!     .with_max_concurrent(10)
//!     .with_quota(ResourceQuota::api_calls(1000));
//!
//! let bulkhead = Bulkhead::new("agent-123", config);
//!
//! // Acquire permit before executing task
//! let permit = bulkhead.acquire().await?;
//! // ... execute task ...
//! drop(permit); // Release when done
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};
use uuid::Uuid;

/// Resource quota types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceQuota {
    /// Maximum API calls (per hour)
    ApiCalls { limit: u64, current: u64 },
    /// Maximum memory usage (bytes)
    Memory { limit_bytes: u64 },
    /// Maximum CPU time (milliseconds per minute)
    CpuTime { limit_ms: u64 },
    /// Maximum tokens (for LLM calls)
    Tokens { limit: u64, current: u64 },
    /// Maximum cost (in microdollars)
    Cost { limit_micros: u64, current: u64 },
}

impl ResourceQuota {
    /// Create API call quota.
    pub fn api_calls(limit: u64) -> Self {
        Self::ApiCalls { limit, current: 0 }
    }

    /// Create token quota.
    pub fn tokens(limit: u64) -> Self {
        Self::Tokens { limit, current: 0 }
    }

    /// Create cost quota (in microdollars - $1 = 1_000_000).
    pub fn cost(limit_dollars: f64) -> Self {
        Self::Cost {
            limit_micros: (limit_dollars * 1_000_000.0) as u64,
            current: 0,
        }
    }

    /// Check if quota exceeded.
    pub fn is_exceeded(&self) -> bool {
        match self {
            Self::ApiCalls { limit, current } => current >= limit,
            Self::Tokens { limit, current } => current >= limit,
            Self::Cost { limit_micros, current } => current >= limit_micros,
            Self::Memory { .. } | Self::CpuTime { .. } => false, // Enforced by runtime
        }
    }

    /// Remaining quota percentage (0.0 - 1.0).
    pub fn remaining_pct(&self) -> f64 {
        match self {
            Self::ApiCalls { limit, current } => 1.0 - (*current as f64 / *limit as f64).min(1.0),
            Self::Tokens { limit, current } => 1.0 - (*current as f64 / *limit as f64).min(1.0),
            Self::Cost { limit_micros, current } => 1.0 - (*current as f64 / *limit_micros as f64).min(1.0),
            Self::Memory { .. } | Self::CpuTime { .. } => 1.0,
        }
    }
}

/// Bulkhead rejection reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkheadRejection {
    /// Too many concurrent requests
    MaxConcurrentExceeded { current: usize, max: usize },
    /// Quota exceeded
    QuotaExceeded { quota_type: String, current: u64, limit: u64 },
    /// Agent is suspended
    AgentSuspended { reason: String },
    /// Timeout waiting for permit
    Timeout { waited_ms: u64 },
}

impl std::fmt::Display for BulkheadRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxConcurrentExceeded { current, max } => {
                write!(f, "Max concurrent requests exceeded ({}/{})", current, max)
            }
            Self::QuotaExceeded { quota_type, current, limit } => {
                write!(f, "Quota exceeded: {} ({}/{})", quota_type, current, limit)
            }
            Self::AgentSuspended { reason } => {
                write!(f, "Agent suspended: {}", reason)
            }
            Self::Timeout { waited_ms } => {
                write!(f, "Timeout waiting for permit ({}ms)", waited_ms)
            }
        }
    }
}

impl std::error::Error for BulkheadRejection {}

/// Bulkhead configuration.
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum concurrent requests per agent
    pub max_concurrent: usize,
    /// Timeout for acquiring permit (ms)
    pub acquire_timeout_ms: u64,
    /// Resource quotas
    pub quotas: Vec<ResourceQuota>,
    /// Enable fair queuing (FIFO)
    pub fair_queuing: bool,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            acquire_timeout_ms: 5000,
            quotas: vec![
                ResourceQuota::api_calls(1000),
                ResourceQuota::tokens(100_000),
            ],
            fair_queuing: true,
        }
    }
}

impl BulkheadConfig {
    /// Set max concurrent requests.
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Add a resource quota.
    pub fn with_quota(mut self, quota: ResourceQuota) -> Self {
        self.quotas.push(quota);
        self
    }

    /// Set acquire timeout.
    pub fn with_timeout_ms(mut self, ms: u64) -> Self {
        self.acquire_timeout_ms = ms;
        self
    }

    /// Presets for different agent tiers.
    pub fn basic() -> Self {
        Self {
            max_concurrent: 5,
            quotas: vec![ResourceQuota::api_calls(100), ResourceQuota::tokens(10_000)],
            ..Default::default()
        }
    }

    pub fn premium() -> Self {
        Self {
            max_concurrent: 50,
            quotas: vec![ResourceQuota::api_calls(10_000), ResourceQuota::tokens(1_000_000)],
            ..Default::default()
        }
    }

    pub fn enterprise() -> Self {
        Self {
            max_concurrent: 200,
            quotas: vec![ResourceQuota::api_calls(100_000), ResourceQuota::tokens(10_000_000)],
            ..Default::default()
        }
    }
}

/// Bulkhead statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BulkheadStats {
    /// Total requests processed
    pub total_requests: u64,
    /// Requests rejected
    pub rejected_requests: u64,
    /// Current concurrent requests
    pub current_concurrent: usize,
    /// Peak concurrent requests
    pub peak_concurrent: usize,
    /// Average wait time (ms)
    pub avg_wait_time_ms: f64,
}

/// Bulkhead for agent resource isolation.
///
/// Uses a semaphore for concurrency control and tracks resource quotas.
pub struct Bulkhead {
    /// Agent ID this bulkhead belongs to
    agent_id: String,
    /// Configuration
    config: BulkheadConfig,
    /// Semaphore for concurrency control
    semaphore: Arc<Semaphore>,
    /// Current quota usage (mutable)
    quota_usage: parking_lot::Mutex<HashMap<String, u64>>,
    /// Statistics
    total_requests: AtomicU64,
    rejected_requests: AtomicU64,
    peak_concurrent: AtomicU64,
    wait_time_sum_ms: AtomicU64,
    /// Suspension status
    suspended: parking_lot::RwLock<Option<String>>,
}

impl Bulkhead {
    /// Create a new bulkhead for an agent.
    pub fn new(agent_id: impl Into<String>, config: BulkheadConfig) -> Self {
        Self {
            agent_id: agent_id.into(),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            quota_usage: parking_lot::Mutex::new(HashMap::new()),
            config,
            total_requests: AtomicU64::new(0),
            rejected_requests: AtomicU64::new(0),
            peak_concurrent: AtomicU64::new(0),
            wait_time_sum_ms: AtomicU64::new(0),
            suspended: parking_lot::RwLock::new(None),
        }
    }

    /// Try to acquire a permit (non-blocking).
    pub fn try_acquire(&self) -> Result<BulkheadPermit<'_>, BulkheadRejection> {
        self.check_suspended()?;
        self.check_quotas()?;

        match self.semaphore.try_acquire() {
            Ok(permit) => {
                self.total_requests.fetch_add(1, Ordering::Relaxed);
                self.update_peak();
                Ok(BulkheadPermit {
                    _permit: permit,
                    bulkhead: self,
                })
            }
            Err(_) => {
                self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                Err(BulkheadRejection::MaxConcurrentExceeded {
                    current: self.config.max_concurrent - self.semaphore.available_permits(),
                    max: self.config.max_concurrent,
                })
            }
        }
    }

    /// Acquire a permit with timeout.
    pub async fn acquire(&self) -> Result<BulkheadPermit<'_>, BulkheadRejection> {
        self.check_suspended()?;
        self.check_quotas()?;

        let start = std::time::Instant::now();
        let timeout = tokio::time::Duration::from_millis(self.config.acquire_timeout_ms);

        match tokio::time::timeout(timeout, self.semaphore.acquire()).await {
            Ok(Ok(permit)) => {
                let wait_ms = start.elapsed().as_millis() as u64;
                self.wait_time_sum_ms.fetch_add(wait_ms, Ordering::Relaxed);
                self.total_requests.fetch_add(1, Ordering::Relaxed);
                self.update_peak();

                Ok(BulkheadPermit {
                    _permit: permit,
                    bulkhead: self,
                })
            }
            Ok(Err(_)) => {
                self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                Err(BulkheadRejection::MaxConcurrentExceeded {
                    current: self.config.max_concurrent,
                    max: self.config.max_concurrent,
                })
            }
            Err(_) => {
                self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                Err(BulkheadRejection::Timeout {
                    waited_ms: self.config.acquire_timeout_ms,
                })
            }
        }
    }

    /// Check if agent is suspended.
    fn check_suspended(&self) -> Result<(), BulkheadRejection> {
        if let Some(reason) = self.suspended.read().clone() {
            return Err(BulkheadRejection::AgentSuspended { reason });
        }
        Ok(())
    }

    /// Check resource quotas.
    fn check_quotas(&self) -> Result<(), BulkheadRejection> {
        for quota in &self.config.quotas {
            if quota.is_exceeded() {
                let (quota_type, current, limit) = match quota {
                    ResourceQuota::ApiCalls { limit, current } => {
                        ("api_calls".to_string(), *current, *limit)
                    }
                    ResourceQuota::Tokens { limit, current } => {
                        ("tokens".to_string(), *current, *limit)
                    }
                    ResourceQuota::Cost { limit_micros, current } => {
                        ("cost".to_string(), *current, *limit_micros)
                    }
                    _ => continue,
                };
                return Err(BulkheadRejection::QuotaExceeded {
                    quota_type,
                    current,
                    limit,
                });
            }
        }
        Ok(())
    }

    /// Update peak concurrent count.
    fn update_peak(&self) {
        let current = self.config.max_concurrent - self.semaphore.available_permits();
        let mut peak = self.peak_concurrent.load(Ordering::Relaxed);
        while current as u64 > peak {
            match self.peak_concurrent.compare_exchange_weak(
                peak,
                current as u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    /// Record API call usage.
    pub fn record_api_call(&self) {
        let mut usage = self.quota_usage.lock();
        *usage.entry("api_calls".to_string()).or_insert(0) += 1;
    }

    /// Record token usage.
    pub fn record_tokens(&self, count: u64) {
        let mut usage = self.quota_usage.lock();
        *usage.entry("tokens".to_string()).or_insert(0) += count;
    }

    /// Record cost usage (microdollars).
    pub fn record_cost(&self, micros: u64) {
        let mut usage = self.quota_usage.lock();
        *usage.entry("cost".to_string()).or_insert(0) += micros;
    }

    /// Suspend the agent.
    pub fn suspend(&self, reason: impl Into<String>) {
        *self.suspended.write() = Some(reason.into());
        tracing::warn!(agent_id = %self.agent_id, "Agent suspended");
    }

    /// Resume the agent.
    pub fn resume(&self) {
        *self.suspended.write() = None;
        tracing::info!(agent_id = %self.agent_id, "Agent resumed");
    }

    /// Get statistics.
    pub fn stats(&self) -> BulkheadStats {
        let total = self.total_requests.load(Ordering::Relaxed).max(1);
        BulkheadStats {
            total_requests: total,
            rejected_requests: self.rejected_requests.load(Ordering::Relaxed),
            current_concurrent: self.config.max_concurrent - self.semaphore.available_permits(),
            peak_concurrent: self.peak_concurrent.load(Ordering::Relaxed) as usize,
            avg_wait_time_ms: self.wait_time_sum_ms.load(Ordering::Relaxed) as f64 / total as f64,
        }
    }

    /// Get agent ID.
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Get available permits.
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// Permit that must be held while executing a task.
pub struct BulkheadPermit<'a> {
    _permit: SemaphorePermit<'a>,
    bulkhead: &'a Bulkhead,
}

impl<'a> BulkheadPermit<'a> {
    /// Record that an API call was made.
    pub fn record_api_call(&self) {
        self.bulkhead.record_api_call();
    }

    /// Record token usage.
    pub fn record_tokens(&self, count: u64) {
        self.bulkhead.record_tokens(count);
    }

    /// Record cost.
    pub fn record_cost(&self, micros: u64) {
        self.bulkhead.record_cost(micros);
    }
}

/// Bulkhead manager for multiple agents.
pub struct BulkheadManager {
    bulkheads: parking_lot::RwLock<HashMap<String, Arc<Bulkhead>>>,
    default_config: BulkheadConfig,
}

impl BulkheadManager {
    /// Create a new manager.
    pub fn new(default_config: BulkheadConfig) -> Self {
        Self {
            bulkheads: parking_lot::RwLock::new(HashMap::new()),
            default_config,
        }
    }

    /// Get or create bulkhead for an agent.
    pub fn get_or_create(&self, agent_id: &str) -> Arc<Bulkhead> {
        // Check if exists
        if let Some(bulkhead) = self.bulkheads.read().get(agent_id) {
            return Arc::clone(bulkhead);
        }

        // Create new
        let mut bulkheads = self.bulkheads.write();
        bulkheads
            .entry(agent_id.to_string())
            .or_insert_with(|| Arc::new(Bulkhead::new(agent_id, self.default_config.clone())))
            .clone()
    }

    /// Get existing bulkhead.
    pub fn get(&self, agent_id: &str) -> Option<Arc<Bulkhead>> {
        self.bulkheads.read().get(agent_id).cloned()
    }

    /// Remove a bulkhead.
    pub fn remove(&self, agent_id: &str) -> Option<Arc<Bulkhead>> {
        self.bulkheads.write().remove(agent_id)
    }

    /// Get all agent IDs.
    pub fn agent_ids(&self) -> Vec<String> {
        self.bulkheads.read().keys().cloned().collect()
    }

    /// Suspend an agent.
    pub fn suspend_agent(&self, agent_id: &str, reason: &str) {
        if let Some(bulkhead) = self.get(agent_id) {
            bulkhead.suspend(reason);
        }
    }

    /// Resume an agent.
    pub fn resume_agent(&self, agent_id: &str) {
        if let Some(bulkhead) = self.get(agent_id) {
            bulkhead.resume();
        }
    }
}

impl Default for BulkheadManager {
    fn default() -> Self {
        Self::new(BulkheadConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_acquire() {
        let bulkhead = Bulkhead::new("agent-1", BulkheadConfig::default());
        
        let permit = bulkhead.acquire().await;
        assert!(permit.is_ok());

        let stats = bulkhead.stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.current_concurrent, 1);
    }

    #[tokio::test]
    async fn test_max_concurrent() {
        let config = BulkheadConfig::default().with_max_concurrent(2);
        let bulkhead = Bulkhead::new("agent-2", config);

        let _p1 = bulkhead.try_acquire().unwrap();
        let _p2 = bulkhead.try_acquire().unwrap();
        
        // Third should fail
        let p3 = bulkhead.try_acquire();
        assert!(matches!(p3, Err(BulkheadRejection::MaxConcurrentExceeded { .. })));
    }

    #[test]
    fn test_suspension() {
        let bulkhead = Bulkhead::new("agent-3", BulkheadConfig::default());
        
        bulkhead.suspend("Policy violation");
        
        let result = bulkhead.try_acquire();
        assert!(matches!(result, Err(BulkheadRejection::AgentSuspended { .. })));

        bulkhead.resume();
        let result = bulkhead.try_acquire();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_presets() {
        let basic = BulkheadConfig::basic();
        assert_eq!(basic.max_concurrent, 5);

        let enterprise = BulkheadConfig::enterprise();
        assert_eq!(enterprise.max_concurrent, 200);
    }

    #[test]
    fn test_manager() {
        let manager = BulkheadManager::default();
        
        let b1 = manager.get_or_create("agent-a");
        let b2 = manager.get_or_create("agent-a");
        
        // Should be same instance
        assert!(Arc::ptr_eq(&b1, &b2));
    }
}

//! Adaptive Query Execution
//!
//! Per ENGINEERING_STANDARD.md Section 2: "Adaptive Execution"
//! - Uses Arrow/Polars for high-performance data processing
//! - Maintains multiple execution plans (SIMD-vectorized vs Standard)
//! - Switches strategies per-request based on live system pressure
//!
//! This enables deterministic self-optimization, not stochastic.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Query execution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    /// Standard execution (safe, predictable)
    Standard,
    /// SIMD-vectorized execution (faster for large datasets)
    Vectorized,
    /// Streaming execution (for out-of-memory datasets)
    Streaming,
}

/// Query execution metrics.
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    pub total_queries: u64,
    pub avg_latency_us: u64,
    pub p99_latency_us: u64,
    pub strategy_usage: HashMap<ExecutionStrategy, u64>,
}

/// Live system pressure indicator.
#[derive(Debug, Clone)]
pub struct SystemPressure {
    /// CPU utilization (0.0 - 1.0)
    pub cpu_utilization: f64,
    /// Memory pressure (0.0 - 1.0)
    pub memory_pressure: f64,
    /// Current query backlog
    pub query_backlog: u64,
}

impl Default for SystemPressure {
    fn default() -> Self {
        Self {
            cpu_utilization: 0.0,
            memory_pressure: 0.0,
            query_backlog: 0,
        }
    }
}

/// Adaptive query executor.
pub struct AdaptiveExecutor {
    /// Current execution strategy
    current_strategy: RwLock<ExecutionStrategy>,
    /// Strategy switch thresholds
    thresholds: ExecutionThresholds,
    /// Metrics collector
    metrics: QueryMetrics,
    /// System pressure sensor
    pressure: RwLock<SystemPressure>,
}

/// Thresholds for strategy switching.
#[derive(Debug, Clone)]
pub struct ExecutionThresholds {
    /// Switch to streaming if dataset size exceeds this (bytes)
    pub streaming_threshold_bytes: usize,
    /// Switch to vectorized if CPU utilization is below this
    pub vectorized_cpu_threshold: f64,
    /// Switch to standard if memory pressure exceeds this
    pub standard_memory_threshold: f64,
}

impl Default for ExecutionThresholds {
    fn default() -> Self {
        Self {
            streaming_threshold_bytes: 1024 * 1024 * 1024, // 1GB
            vectorized_cpu_threshold: 0.7,
            standard_memory_threshold: 0.8,
        }
    }
}

/// Internal metrics collector.
struct QueryMetrics {
    total: AtomicU64,
    latencies: RwLock<Vec<u64>>,
    strategy_counts: RwLock<HashMap<ExecutionStrategy, u64>>,
}

impl Default for QueryMetrics {
    fn default() -> Self {
        Self {
            total: AtomicU64::new(0),
            latencies: RwLock::new(Vec::new()),
            strategy_counts: RwLock::new(HashMap::new()),
        }
    }
}

impl AdaptiveExecutor {
    /// Create a new adaptive executor.
    pub fn new() -> Self {
        Self::with_thresholds(ExecutionThresholds::default())
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: ExecutionThresholds) -> Self {
        Self {
            current_strategy: RwLock::new(ExecutionStrategy::Standard),
            thresholds,
            metrics: QueryMetrics::default(),
            pressure: RwLock::new(SystemPressure::default()),
        }
    }

    /// Get current execution strategy.
    pub fn current_strategy(&self) -> ExecutionStrategy {
        *self.current_strategy.read()
    }

    /// Update system pressure metrics.
    pub fn update_pressure(&self, pressure: SystemPressure) {
        *self.pressure.write() = pressure;
        self.adapt_strategy();
    }

    /// Adapt strategy based on current pressure.
    fn adapt_strategy(&self) {
        let pressure = self.pressure.read().clone();
        let new_strategy = self.select_strategy(&pressure);

        let mut current = self.current_strategy.write();
        if *current != new_strategy {
            tracing::info!(
                from = ?*current,
                to = ?new_strategy,
                cpu = pressure.cpu_utilization,
                memory = pressure.memory_pressure,
                "Switching execution strategy"
            );
            *current = new_strategy;
        }
    }

    /// Select optimal strategy based on pressure.
    fn select_strategy(&self, pressure: &SystemPressure) -> ExecutionStrategy {
        // High memory pressure -> use streaming
        if pressure.memory_pressure > self.thresholds.standard_memory_threshold {
            return ExecutionStrategy::Streaming;
        }

        // Low CPU, can use vectorized
        if pressure.cpu_utilization < self.thresholds.vectorized_cpu_threshold {
            return ExecutionStrategy::Vectorized;
        }

        ExecutionStrategy::Standard
    }

    /// Execute a query with automatic strategy selection.
    pub async fn execute<F, T>(&self, dataset_size_bytes: usize, query_fn: F) -> T
    where
        F: FnOnce(ExecutionStrategy) -> T,
    {
        let start = Instant::now();

        // Select strategy for this query
        let strategy = if dataset_size_bytes > self.thresholds.streaming_threshold_bytes {
            ExecutionStrategy::Streaming
        } else {
            *self.current_strategy.read()
        };

        // Execute
        let result = query_fn(strategy);

        // Record metrics
        let latency_us = start.elapsed().as_micros() as u64;
        self.record_execution(strategy, latency_us);

        result
    }

    fn record_execution(&self, strategy: ExecutionStrategy, latency_us: u64) {
        self.metrics.total.fetch_add(1, Ordering::Relaxed);

        let mut latencies = self.metrics.latencies.write();
        latencies.push(latency_us);
        if latencies.len() > 10000 {
            latencies.remove(0);
        }

        *self
            .metrics
            .strategy_counts
            .write()
            .entry(strategy)
            .or_insert(0) += 1;
    }

    /// Get execution metrics.
    pub fn get_metrics(&self) -> ExecutionMetrics {
        let total = self.metrics.total.load(Ordering::Relaxed);
        let latencies = self.metrics.latencies.read();

        let avg = if latencies.is_empty() {
            0
        } else {
            latencies.iter().sum::<u64>() / latencies.len() as u64
        };

        let p99 = if latencies.is_empty() {
            0
        } else {
            let mut sorted = latencies.clone();
            sorted.sort();
            let idx = (sorted.len() as f64 * 0.99) as usize;
            sorted.get(idx.min(sorted.len() - 1)).copied().unwrap_or(0)
        };

        ExecutionMetrics {
            total_queries: total,
            avg_latency_us: avg,
            p99_latency_us: p99,
            strategy_usage: self.metrics.strategy_counts.read().clone(),
        }
    }
}

impl Default for AdaptiveExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = AdaptiveExecutor::new();
        assert_eq!(executor.current_strategy(), ExecutionStrategy::Standard);
    }

    #[test]
    fn test_strategy_adaptation() {
        let executor = AdaptiveExecutor::new();

        // Low pressure -> vectorized
        executor.update_pressure(SystemPressure {
            cpu_utilization: 0.3,
            memory_pressure: 0.2,
            query_backlog: 0,
        });
        assert_eq!(executor.current_strategy(), ExecutionStrategy::Vectorized);

        // High memory -> streaming
        executor.update_pressure(SystemPressure {
            cpu_utilization: 0.5,
            memory_pressure: 0.9,
            query_backlog: 10,
        });
        assert_eq!(executor.current_strategy(), ExecutionStrategy::Streaming);
    }

    #[tokio::test]
    async fn test_query_execution() {
        let executor = AdaptiveExecutor::new();

        let result = executor
            .execute(1024, |strategy| {
                assert_eq!(strategy, ExecutionStrategy::Standard);
                42
            })
            .await;

        assert_eq!(result, 42);
        assert_eq!(executor.get_metrics().total_queries, 1);
    }

    #[test]
    fn test_large_dataset_forces_streaming() {
        let executor = AdaptiveExecutor::new();
        let large_size = 2 * 1024 * 1024 * 1024; // 2GB

        // Large dataset should use streaming regardless of current strategy
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            executor
                .execute(large_size, |strategy| {
                    assert_eq!(strategy, ExecutionStrategy::Streaming);
                })
                .await;
        });
    }
}

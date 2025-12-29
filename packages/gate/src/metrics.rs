//! Production Metrics for Gate Modules
//!
//! Exports Prometheus-compatible metrics for:
//! - WasmRegistry (module loads, invocations, latency)
//! - ContextGuard (scans, flagged chunks)
//! - PromptGuard (analysis count, threat levels)

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global metrics registry.
pub static METRICS: once_cell::sync::Lazy<GateMetricsExporter> = 
    once_cell::sync::Lazy::new(GateMetricsExporter::new);

/// Gate module metrics exporter.
pub struct GateMetricsExporter {
    // WASM Registry metrics
    wasm_modules_loaded: AtomicU64,
    wasm_modules_unloaded: AtomicU64,
    wasm_invocations_total: AtomicU64,
    wasm_invocation_latency_us: RwLock<Vec<u64>>,
    
    // Context Guard metrics
    context_scans_total: AtomicU64,
    context_chunks_scanned: AtomicU64,
    context_chunks_flagged: AtomicU64,
    context_rejected_total: AtomicU64,
    
    // Prompt Guard metrics
    prompt_analyses_total: AtomicU64,
    prompt_blocked_total: AtomicU64,
    prompt_threat_levels: RwLock<HashMap<String, u64>>,
    prompt_latency_us: RwLock<Vec<u64>>,
}

impl GateMetricsExporter {
    pub fn new() -> Self {
        Self {
            wasm_modules_loaded: AtomicU64::new(0),
            wasm_modules_unloaded: AtomicU64::new(0),
            wasm_invocations_total: AtomicU64::new(0),
            wasm_invocation_latency_us: RwLock::new(Vec::new()),
            
            context_scans_total: AtomicU64::new(0),
            context_chunks_scanned: AtomicU64::new(0),
            context_chunks_flagged: AtomicU64::new(0),
            context_rejected_total: AtomicU64::new(0),
            
            prompt_analyses_total: AtomicU64::new(0),
            prompt_blocked_total: AtomicU64::new(0),
            prompt_threat_levels: RwLock::new(HashMap::new()),
            prompt_latency_us: RwLock::new(Vec::new()),
        }
    }

    // ========== WASM Registry Metrics ==========
    
    pub fn record_wasm_load(&self) {
        self.wasm_modules_loaded.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_wasm_unload(&self) {
        self.wasm_modules_unloaded.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_wasm_invocation(&self, latency_us: u64) {
        self.wasm_invocations_total.fetch_add(1, Ordering::Relaxed);
        let mut latencies = self.wasm_invocation_latency_us.write();
        latencies.push(latency_us);
        // Keep last 1000 samples
        if latencies.len() > 1000 {
            latencies.remove(0);
        }
    }
    
    // ========== Context Guard Metrics ==========
    
    pub fn record_context_scan(&self, chunks_scanned: u64, chunks_flagged: u64, rejected: bool) {
        self.context_scans_total.fetch_add(1, Ordering::Relaxed);
        self.context_chunks_scanned.fetch_add(chunks_scanned, Ordering::Relaxed);
        self.context_chunks_flagged.fetch_add(chunks_flagged, Ordering::Relaxed);
        if rejected {
            self.context_rejected_total.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    // ========== Prompt Guard Metrics ==========
    
    pub fn record_prompt_analysis(&self, threat_level: &str, blocked: bool, latency_us: u64) {
        self.prompt_analyses_total.fetch_add(1, Ordering::Relaxed);
        if blocked {
            self.prompt_blocked_total.fetch_add(1, Ordering::Relaxed);
        }
        
        {
            let mut levels = self.prompt_threat_levels.write();
            *levels.entry(threat_level.to_string()).or_insert(0) += 1;
        }
        
        {
            let mut latencies = self.prompt_latency_us.write();
            latencies.push(latency_us);
            if latencies.len() > 1000 {
                latencies.remove(0);
            }
        }
    }
    
    // ========== Export Methods ==========
    
    /// Export metrics in Prometheus text format.
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        // WASM metrics
        output.push_str(&format!(
            "# HELP gate_wasm_modules_loaded_total Total WASM modules loaded\n\
             # TYPE gate_wasm_modules_loaded_total counter\n\
             gate_wasm_modules_loaded_total {}\n\n",
            self.wasm_modules_loaded.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP gate_wasm_invocations_total Total WASM invocations\n\
             # TYPE gate_wasm_invocations_total counter\n\
             gate_wasm_invocations_total {}\n\n",
            self.wasm_invocations_total.load(Ordering::Relaxed)
        ));
        
        // WASM latency histogram
        let wasm_latencies = self.wasm_invocation_latency_us.read();
        if !wasm_latencies.is_empty() {
            let avg = wasm_latencies.iter().sum::<u64>() / wasm_latencies.len() as u64;
            output.push_str(&format!(
                "# HELP gate_wasm_invocation_latency_us WASM invocation latency\n\
                 # TYPE gate_wasm_invocation_latency_us gauge\n\
                 gate_wasm_invocation_latency_us {}\n\n",
                avg
            ));
        }
        
        // Context Guard metrics
        output.push_str(&format!(
            "# HELP gate_context_scans_total Total RAG context scans\n\
             # TYPE gate_context_scans_total counter\n\
             gate_context_scans_total {}\n\n",
            self.context_scans_total.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP gate_context_chunks_flagged_total Flagged RAG chunks\n\
             # TYPE gate_context_chunks_flagged_total counter\n\
             gate_context_chunks_flagged_total {}\n\n",
            self.context_chunks_flagged.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP gate_context_rejected_total Rejected RAG contexts\n\
             # TYPE gate_context_rejected_total counter\n\
             gate_context_rejected_total {}\n\n",
            self.context_rejected_total.load(Ordering::Relaxed)
        ));
        
        // Prompt Guard metrics
        output.push_str(&format!(
            "# HELP gate_prompt_analyses_total Total prompt analyses\n\
             # TYPE gate_prompt_analyses_total counter\n\
             gate_prompt_analyses_total {}\n\n",
            self.prompt_analyses_total.load(Ordering::Relaxed)
        ));
        
        output.push_str(&format!(
            "# HELP gate_prompt_blocked_total Blocked prompts\n\
             # TYPE gate_prompt_blocked_total counter\n\
             gate_prompt_blocked_total {}\n\n",
            self.prompt_blocked_total.load(Ordering::Relaxed)
        ));
        
        // Threat level breakdown
        let levels = self.prompt_threat_levels.read();
        for (level, count) in levels.iter() {
            output.push_str(&format!(
                "gate_prompt_threat_level{{level=\"{}\"}} {}\n",
                level, count
            ));
        }
        
        output
    }
    
    /// Get summary statistics.
    pub fn summary(&self) -> MetricsSummary {
        let wasm_latencies = self.wasm_invocation_latency_us.read();
        let prompt_latencies = self.prompt_latency_us.read();
        
        MetricsSummary {
            wasm_modules_loaded: self.wasm_modules_loaded.load(Ordering::Relaxed),
            wasm_invocations: self.wasm_invocations_total.load(Ordering::Relaxed),
            wasm_avg_latency_us: if wasm_latencies.is_empty() { 0 } else {
                wasm_latencies.iter().sum::<u64>() / wasm_latencies.len() as u64
            },
            context_scans: self.context_scans_total.load(Ordering::Relaxed),
            context_flagged: self.context_chunks_flagged.load(Ordering::Relaxed),
            prompt_analyses: self.prompt_analyses_total.load(Ordering::Relaxed),
            prompt_blocked: self.prompt_blocked_total.load(Ordering::Relaxed),
            prompt_avg_latency_us: if prompt_latencies.is_empty() { 0 } else {
                prompt_latencies.iter().sum::<u64>() / prompt_latencies.len() as u64
            },
        }
    }
    
    /// Reset all metrics (for testing).
    pub fn reset(&self) {
        self.wasm_modules_loaded.store(0, Ordering::Relaxed);
        self.wasm_modules_unloaded.store(0, Ordering::Relaxed);
        self.wasm_invocations_total.store(0, Ordering::Relaxed);
        self.wasm_invocation_latency_us.write().clear();
        
        self.context_scans_total.store(0, Ordering::Relaxed);
        self.context_chunks_scanned.store(0, Ordering::Relaxed);
        self.context_chunks_flagged.store(0, Ordering::Relaxed);
        self.context_rejected_total.store(0, Ordering::Relaxed);
        
        self.prompt_analyses_total.store(0, Ordering::Relaxed);
        self.prompt_blocked_total.store(0, Ordering::Relaxed);
        self.prompt_threat_levels.write().clear();
        self.prompt_latency_us.write().clear();
    }
}

impl Default for GateMetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of gate metrics.
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub wasm_modules_loaded: u64,
    pub wasm_invocations: u64,
    pub wasm_avg_latency_us: u64,
    pub context_scans: u64,
    pub context_flagged: u64,
    pub prompt_analyses: u64,
    pub prompt_blocked: u64,
    pub prompt_avg_latency_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_metrics() {
        let metrics = GateMetricsExporter::new();
        
        metrics.record_wasm_load();
        metrics.record_wasm_load();
        metrics.record_wasm_invocation(100);
        metrics.record_wasm_invocation(200);
        
        let summary = metrics.summary();
        assert_eq!(summary.wasm_modules_loaded, 2);
        assert_eq!(summary.wasm_invocations, 2);
        assert_eq!(summary.wasm_avg_latency_us, 150);
    }
    
    #[test]
    fn test_context_metrics() {
        let metrics = GateMetricsExporter::new();
        
        metrics.record_context_scan(10, 2, false);
        metrics.record_context_scan(5, 0, false);
        metrics.record_context_scan(8, 3, true);
        
        let summary = metrics.summary();
        assert_eq!(summary.context_scans, 3);
        assert_eq!(summary.context_flagged, 5);
    }
    
    #[test]
    fn test_prometheus_export() {
        let metrics = GateMetricsExporter::new();
        
        metrics.record_wasm_load();
        metrics.record_prompt_analysis("High", true, 500);
        
        let output = metrics.export_prometheus();
        
        assert!(output.contains("gate_wasm_modules_loaded_total 1"));
        assert!(output.contains("gate_prompt_blocked_total 1"));
    }
    
    #[test]
    fn test_reset() {
        let metrics = GateMetricsExporter::new();
        
        metrics.record_wasm_load();
        metrics.record_context_scan(10, 2, false);
        
        metrics.reset();
        
        let summary = metrics.summary();
        assert_eq!(summary.wasm_modules_loaded, 0);
        assert_eq!(summary.context_scans, 0);
    }
}

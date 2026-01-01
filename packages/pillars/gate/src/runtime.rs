//! Native Tokio io_uring Runtime
//!
//! Per ARCHITECTURE.md: "The Hyper-Loop"
//! Uses native tokio-uring (will be absorbed into Tokio core as it stabilizes)
//!
//! Why tokio-uring over alternatives:
//! - Tokio 1.48.0+ has unstable io_uring support in core
//! - tokio-uring is the official Tokio project for io_uring
//! - Will eventually merge into main Tokio
//! - No ecosystem friction (already using Tokio everywhere)

use std::future::Future;

/// Runtime configuration for io_uring.
#[derive(Debug, Clone)]
pub struct IoUringRuntimeConfig {
    /// Number of entries in the io_uring submission queue
    pub sq_entries: u32,
    /// Number of entries in the io_uring completion queue
    pub cq_entries: u32,
    /// Enable kernel polling (IORING_SETUP_SQPOLL)
    pub kernel_poll: bool,
}

impl Default for IoUringRuntimeConfig {
    fn default() -> Self {
        Self {
            sq_entries: 128,
            cq_entries: 256,
            kernel_poll: false, // Requires root or CAP_SYS_NICE
        }
    }
}

/// Native Tokio io_uring runtime.
/// On Linux with io_uring feature, uses tokio-uring.
/// Otherwise, falls back to standard Tokio.
#[cfg(all(target_os = "linux", feature = "io_uring"))]
pub mod uring {
    use super::*;

    /// Run a future on the tokio-uring runtime.
    /// This is the zero-copy, high-performance path.
    pub fn start<F: Future>(future: F) -> F::Output {
        tokio_uring::start(future)
    }

    /// Spawn a future on the current tokio-uring runtime.
    pub fn spawn<F>(future: F) -> tokio_uring::task::JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        tokio_uring::spawn(future)
    }

    /// Open a file with io_uring.
    pub async fn open_file(
        path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<tokio_uring::fs::File> {
        tokio_uring::fs::File::open(path).await
    }

    /// Create a file with io_uring.
    pub async fn create_file(
        path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<tokio_uring::fs::File> {
        tokio_uring::fs::File::create(path).await
    }
}

// ============================================================================
// Fallback runtime (non-Linux or io_uring disabled)
// ============================================================================

/// Tokio-based async runtime (fallback for non-Linux).
pub struct TokioRuntime {
    runtime: tokio::runtime::Runtime,
}

impl TokioRuntime {
    /// Create a new multi-threaded Tokio runtime.
    pub fn new() -> std::io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        Ok(Self { runtime })
    }

    /// Run a future to completion.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// Spawn a future on the runtime.
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }
}

impl Default for TokioRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create Tokio runtime")
    }
}

/// Unified runtime that uses io_uring on Linux, Tokio elsewhere.
pub struct HyperRuntime {
    config: IoUringRuntimeConfig,
}

impl HyperRuntime {
    pub fn new() -> Self {
        Self::with_config(IoUringRuntimeConfig::default())
    }

    pub fn with_config(config: IoUringRuntimeConfig) -> Self {
        Self { config }
    }

    /// Run a future on the best available runtime.
    #[cfg(all(target_os = "linux", feature = "io_uring"))]
    pub fn run<F: Future>(future: F) -> F::Output {
        uring::start(future)
    }

    #[cfg(not(all(target_os = "linux", feature = "io_uring")))]
    pub fn run<F: Future>(future: F) -> F::Output {
        let rt = TokioRuntime::new().expect("Failed to create runtime");
        rt.block_on(future)
    }

    pub fn config(&self) -> &IoUringRuntimeConfig {
        &self.config
    }

    /// Check if io_uring is available.
    pub fn is_io_uring_available() -> bool {
        #[cfg(all(target_os = "linux", feature = "io_uring"))]
        {
            true
        }
        #[cfg(not(all(target_os = "linux", feature = "io_uring")))]
        {
            false
        }
    }
}

impl Default for HyperRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = IoUringRuntimeConfig::default();
        assert_eq!(config.sq_entries, 128);
        assert_eq!(config.cq_entries, 256);
    }

    #[test]
    fn test_hyper_runtime_creation() {
        let runtime = HyperRuntime::new();
        assert_eq!(runtime.config().sq_entries, 128);
    }

    #[test]
    fn test_tokio_runtime() {
        let rt = TokioRuntime::new().unwrap();
        let result = rt.block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_io_uring_detection() {
        let available = HyperRuntime::is_io_uring_available();
        #[cfg(all(target_os = "linux", feature = "io_uring"))]
        assert!(available);
        #[cfg(not(all(target_os = "linux", feature = "io_uring")))]
        assert!(!available);
    }
}

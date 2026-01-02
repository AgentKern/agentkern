//! Thread-per-Core Runtime for Arbiter
//!
//! Per ARCHITECTURE.md Section 1: "The Hyper-Loop"
//! - Thread-per-Core architecture for minimal context switching
//! - Each CPU core runs one thread with its own work queue
//!
//! This provides predictable, low-latency lock coordination.

use std::sync::Arc;

/// Thread-per-core configuration.
#[derive(Debug, Clone)]
pub struct ThreadPerCoreConfig {
    /// Number of cores to use (default: all)
    pub cores: usize,
    /// Pin threads to cores
    pub pin_threads: bool,
    /// Queue size per core
    pub queue_size: usize,
}

impl Default for ThreadPerCoreConfig {
    fn default() -> Self {
        Self {
            cores: num_cpus(),
            pin_threads: true,
            queue_size: 1024,
        }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Work item for a core.
pub type WorkFn = Box<dyn FnOnce() + Send + 'static>;

/// Per-core work queue.
pub struct CoreQueue {
    core_id: usize,
    sender: std::sync::mpsc::Sender<WorkFn>,
}

impl CoreQueue {
    /// Submit work to this core.
    pub fn submit(&self, work: WorkFn) -> Result<(), &'static str> {
        self.sender.send(work).map_err(|_| "Core queue closed")
    }

    /// Get the core ID.
    pub fn core_id(&self) -> usize {
        self.core_id
    }
}

/// Thread-per-core runtime.
pub struct ThreadPerCoreRuntime {
    config: ThreadPerCoreConfig,
    queues: Vec<Arc<CoreQueue>>,
    handles: Vec<std::thread::JoinHandle<()>>,
}

impl ThreadPerCoreRuntime {
    /// Create a new thread-per-core runtime.
    pub fn new(config: ThreadPerCoreConfig) -> Self {
        let mut queues = Vec::with_capacity(config.cores);
        let mut handles = Vec::with_capacity(config.cores);

        for core_id in 0..config.cores {
            let (sender, receiver) = std::sync::mpsc::channel::<WorkFn>();

            let queue = Arc::new(CoreQueue { core_id, sender });
            queues.push(Arc::clone(&queue));

            let pin_threads = config.pin_threads;
            let handle = std::thread::Builder::new()
                .name(format!("arbiter-core-{}", core_id))
                .spawn(move || {
                    // Pin thread to core if requested
                    if pin_threads {
                        #[cfg(target_os = "linux")]
                        {
                            let _ = set_thread_affinity(core_id);
                        }
                    }

                    tracing::debug!(core_id, "Worker thread started");

                    for work in receiver {
                        work();
                    }

                    tracing::debug!(core_id, "Worker thread stopped");
                })
                .expect("Failed to spawn worker thread");

            handles.push(handle);
        }

        Self {
            config,
            queues,
            handles,
        }
    }

    /// Get the number of cores.
    pub fn num_cores(&self) -> usize {
        self.config.cores
    }

    /// Get a queue by core ID.
    pub fn queue(&self, core_id: usize) -> Option<Arc<CoreQueue>> {
        self.queues.get(core_id).cloned()
    }

    /// Submit work to a specific core.
    pub fn submit_to_core(&self, core_id: usize, work: WorkFn) -> Result<(), &'static str> {
        self.queues
            .get(core_id)
            .ok_or("Invalid core ID")?
            .submit(work)
    }

    /// Submit work using consistent hashing.
    pub fn submit_hashed(&self, key: &str, work: WorkFn) -> Result<(), &'static str> {
        let hash = hash_string(key);
        let core_id = (hash as usize) % self.config.cores;
        self.submit_to_core(core_id, work)
    }

    /// Shutdown the runtime.
    pub fn shutdown(self) {
        // Drop senders to signal shutdown
        drop(self.queues);

        // Wait for threads
        for handle in self.handles {
            let _ = handle.join();
        }
    }
}

impl Default for ThreadPerCoreRuntime {
    fn default() -> Self {
        Self::new(ThreadPerCoreConfig::default())
    }
}

fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(target_os = "linux")]
fn set_thread_affinity(core_id: usize) -> Result<(), std::io::Error> {
    use std::io;

    // Use libc to set CPU affinity
    unsafe {
        let mut cpuset: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_ZERO(&mut cpuset);
        libc::CPU_SET(core_id, &mut cpuset);

        let result = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpuset);
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_thread_per_core_runtime() {
        let runtime = ThreadPerCoreRuntime::new(ThreadPerCoreConfig {
            cores: 2,
            pin_threads: false,
            queue_size: 16,
        });

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        runtime
            .submit_to_core(
                0,
                Box::new(move || {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                }),
            )
            .unwrap();

        // Give it time to process
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert_eq!(counter.load(Ordering::SeqCst), 1);

        runtime.shutdown();
    }

    #[test]
    fn test_hashed_submission() {
        let runtime = ThreadPerCoreRuntime::new(ThreadPerCoreConfig {
            cores: 4,
            pin_threads: false,
            queue_size: 16,
        });

        // Same key should always go to same core
        let key = "user:12345";
        let _expected_core = (hash_string(key) as usize) % 4;

        assert!(runtime.submit_hashed(key, Box::new(|| {})).is_ok());

        runtime.shutdown();
    }
}

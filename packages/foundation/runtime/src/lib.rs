#![warn(dead_code)]  // Production: warn on dead code
#![warn(unused)]  // Production: warn on unused code
//! AgentKern Universal Runtime
//!
//! Single binary that runs anywhere:
//! - WASM Components (Primary) - Nano-Light isolation per ARCHITECTURE.md
//! - Container (Fallback) - Only if WASM unavailable
//! - Bare metal, Edge devices, Browser (WASM)
//!
//! No vendor-specific code. Auto-detects and adapts.
//!
//! Per ARCHITECTURE.md: "WASM Components (Nano-Light)" NOT "Docker (Heavy)"

pub mod config;
pub mod detect;
pub mod fallback;
pub mod isolation;
pub mod serve;

pub use config::{auto_configure, RuntimeConfig};
pub use detect::{detect_environment, Environment};
pub use fallback::{FallbackResult, GracefulFallback, ServiceMode};
pub use isolation::{detect_best_isolation, IsolationConfig, IsolationMode};
pub use serve::{serve, Protocol};

/// AgentKern kernel version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run AgentKern with auto-detection.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Detect environment
    let env = detect_environment();
    tracing::info!("Detected environment: {:?}", env);

    // 2. Detect best isolation (WASM preferred)
    let isolation = detect_best_isolation();
    tracing::info!("Isolation mode: {:?}", isolation);

    // 3. Auto-configure based on environment
    let config = auto_configure(&env);
    tracing::info!("Configuration: {:?}", config);

    // 4. Start serving
    serve(&config).await?;

    Ok(())
}

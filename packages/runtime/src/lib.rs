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

pub mod detect;
pub mod config;
pub mod serve;
pub mod isolation;
pub mod fallback;

pub use detect::{Environment, detect_environment};
pub use config::{RuntimeConfig, auto_configure};
pub use serve::{serve, Protocol};
pub use isolation::{IsolationMode, IsolationConfig, detect_best_isolation};
pub use fallback::{ServiceMode, GracefulFallback, FallbackResult};


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

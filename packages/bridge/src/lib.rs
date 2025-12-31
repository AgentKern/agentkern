#![deny(clippy::all)]

use napi_derive::napi;
use agentkern_gate::tee::Enclave;

#[napi]
pub fn attest(nonce: String) -> String {
    // Determine platform (Simulated for now in dev, but using the REAL Rust code)
    // The `Enclave::new` or `Enclave::simulated` logic handles detection.
    
    // For safety in this bridge, we default to simulated if not in a real TEE environment,
    // but we use the ACTUAL verification logic.
    
    // In a real deployment, we would check for /dev/tdx-guest
    let enclave = if std::path::Path::new("/dev/tdx-guest").exists() || std::path::Path::new("/dev/sev-guest").exists() {
         Enclave::new("agentkern-gateway").unwrap_or_else(|_| Enclave::simulated("fallback-sim"))
    } else {
         Enclave::simulated("sim-gateway")
    };

    match enclave.attest(nonce.as_bytes()) {
        Ok(attestation) => attestation.to_json(),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

# Changelog

All notable changes to the "AgentKern" Autonomous Agent Operating System.

## [0.1.0-rc1] - 2026-01-12

### Added
- **Neural Integrity**: Implemented `ModelProvenance` to verify ONNX model signatures (Phase 10).
- **Chaos Engineering**: Integration of `ChaosProxy` for fault injection and `golden_chaos` verification test (Phase 9/10).
- **Self-Healing**: `MeshOrchestrator` now autonomously heals degraded agents via local state restoration (Phase 9).
- **Observability**: Added `cold_start_micros` metrics for WASM/Agent instantiation (Phase 10).
- **Runbooks**: Automated recovery procedures documented in `RUNBOOKS.md`.
- **Global Mesh**: Cross-cloud "Sovereign Mesh" with strict geo-fencing (Phase 7).
- **Enterprise Connectors**: SAP (RFC) and SWIFT (ISO 20022) connectors (Phase 6).

### Security
- **Crypto-Agility**: Upgraded to AES-256-GCM and prepared hybrid Post-Quantum Cryptography hooks (Phase 8).
- **Supply Chain**: Dependency audit completed; `rust_decimal` vulnerability mitigated.
- **Fuzzing**: Added fuzz targets for `iso20022` parser.

### Fixed
- **Race Condition**: Fixed flaky `ee/sovereign-mesh` tests using mutex serialization.
- **API Ownership**: Resolved `check_and_migrate` passport consumption bug.

### Changed
- **Architecture**: Transitioned to "Neural-Symbolic" hybrid core with hardened IO_uring runtime.
- **Licensing**: Enterprise Edition (EE) modules isolated under `ee/` directory.

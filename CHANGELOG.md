# Changelog

All notable changes to the "AgentKern" Autonomous Agent Operating System.

## [0.1.0] - 2026-02-07

### Added
- **Global Strategic Bifurcation**: Clear separation of Open Source (OSS) and Enterprise Edition (EE) codebases.
- **Enterprise Asset Quarantine**: Relocated 19+ premium modules (Mainframe, SAP, SWIFT, Multi-tenancy) to external backup.
- **Pillar Stabilization**: Hardened Gate, Arbiter, Synapse, and Nexus for production OSS release.
- **Technical Gating**: Stubbed premium features (Anti-fragile healing, DR Scheduler, Sandbox) within OSS pillars.
- **Commercial Strategy**: Formalized "Sovereign Control Plane" model in doc.
- **Open Source Setup**: Complete OSS infrastructure (LICENSE, CONTRIBUTING.md, release workflows)
- **Release Automation**: GitHub Actions workflow for automated releases
- **OSS Documentation**: Comprehensive guides for OSS structure and enterprise integration
- **Enterprise Integration Path**: Clear separation and integration guide for enterprise features

### Changed
- **Architecture**: Migrated to Rust-first architecture with unified HTTP server
- **Runtime Model**: Consolidated around the unified Rust server and Rust pillar crates
- **CI/CD**: Updated workflows to focus on Rust server and SDK/API validation
- **Documentation**: Consolidated architecture docs, removed duplicates

### Fixed
- **Code Quality**: All Rust code compiles and tests pass
- **Workspace Configuration**: Updated to exclude enterprise directory from OSS
- **Build System**: Simplified to Rust-only core with optional enterprise extensions

### Documentation
- Created `LICENSE` - Apache 2.0 license
- Created `CONTRIBUTING.md` - Contribution guidelines
- Created `NOTICE` - Attribution and third-party notices
- Created `docs/OSS_SETUP.md` - OSS structure and enterprise integration guide
- Created `docs/RELEASE_PROCESS.md` - Release and versioning guide
- Created `OSS_READY.md` - OSS readiness checklist
- Updated `docs/governance/EPISTEMIC_HEALTH.md` - Architecture status
- Marked outdated ADRs as SUPERSEDED

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

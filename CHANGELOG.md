# Changelog

All notable changes to AgentKern will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-04

### Added

#### SDK Infrastructure
- **SDK Core (Rust)**: Production-ready agent cryptography library
  - `Agent` struct with Ed25519 keypair management (ring crate)
  - `LiabilityProof` JWT creation and verification
  - `A2AMessage` protocol encoding
  - 27 unit tests with 100% pass rate
- **SDK Node.js**: Native bindings via N-API (`@agentkern/sdk`)
  - TypeScript type definitions
  - Agent, LiabilityProof, and A2A exports
- **SDK Python**: Native bindings via PyO3 (`agentkern`)
  - Type stubs (.pyi) for IDE support
  - Maturin-based wheel packaging

#### Security Enhancements
- **Identity Service**:
  - `LiabilityProofGuard` for X-AgentKernIdentity validation
  - `OptionalAuthGuard` for flexible authentication
  - `CsrfMiddleware` with double-submit cookie pattern
  - 15 comprehensive Semgrep TypeScript security rules
- **Enterprise Edition**:
  - AES-256-GCM encryption in `sovereign-memory`
  - AWS KMS integration with async operations
  - Safe test environment variable handling (`temp_env`)
- **CI/CD**: Security scan workflow (SAST, secrets, dependencies)

#### Documentation
- `SDK_DESIGN.md`: Polyglot SDK architecture (UniFFI strategy)
- `CSRF_INTEGRATION_GUIDE.md`: Client integration examples
- `SECURITY_GUARDS.md`: Authentication guard reference

### Fixed
- **Dependencies**: `qs` vulnerability (HIGH) via pnpm override
- **Enterprise Edition**: 19 modules now compile successfully
  - Integrated 9 new Rust crates into workspace
  - Fixed `ee/idp` demo test (variable name)
- **Testing**: Replaced unsafe environment manipulation with `temp_env`

### Changed
- **Workspace**: Added `sdk-core`, `sdk-node`, `sdks/python` packages
- **Crypto**: Production-grade ring-based Ed25519 (AWS libcrypto)

## [0.1.0] - 2026-01-03

### Added
- Initial release with 6 pillars (Identity, Gate, Synapse, Arbiter, Treasury, Nexus)
- TypeScript Identity service with TypeORM persistence
- Rust N-API bridge for all pillars
- Playground UI with trust score visualizations
- GitHub Pages deployment
- E2E test suite (67 tests)

[0.2.0]: https://github.com/agentkern/agentkern/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/agentkern/agentkern/releases/tag/v0.1.0

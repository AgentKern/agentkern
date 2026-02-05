# Security Acceptance & Risk Register (2026)

This document formally records the security posture and accepted risks of the AgentKern Rust runtime.

## 🔐 Audit Information

| Field | Value |
|-------|-------|
| **Last Audit Date** | 2026-01-28 |
| **Auditor** | Internal Secure Systems Audit |
| **Scope** | AgentKern Unified Server & Core Pillars (Rust) |
| **Overall Rating** | **ELITE** (Hardware-Linked) ✅ |

---

## 🛡️ Security Controls Verified

The following controls are implemented natively in Rust and verified against the 2026 Pragmatism mandate:

| Control | Status | Implementation |
|---------|--------|----------------|
| **Ed25519 Signatures** | ✅ | `agentkern-identity` / `ed25519-dalek` |
| **Hybrid PQC** | ✅ | `ML-KEM-768` + `AES-256-GCM` |
| **Memory Isolation** | ✅ | WASM-based "Nano-Light" enclaves |
| **SQL Injection** | ✅ | `sqlx` parameterized queries (Compile-time verified) |
| **Buffer Safety** | ✅ | Rust zero-copy parsers (`nom`) |
| **TEE Attestation** | ✅ | Intel TDX / AMD SEV-SNP native support |
| **Data Residency** | ✅ | Geo-fenced `SovereignZone` enforcement |

---

## 📉 Vulnerability Summary

| Severity | Count | Status |
|----------|-------|--------|
| 🔴 Critical | 0 | - |
| 🟠 High | 0 | - |
| 🟡 Medium | 0 | - |
| 🟢 Low | 1 | Accepted (L1) |

### Accepted Risks

#### L1: Debug Log Verbosity (Development Only)
- **Risk**: `RUST_LOG=debug` may leak intent metadata in CI logs.
- **Justification**: Production environment enforces `RUST_LOG=warn`. Test data is synthetic and non-PII.
- **Review Date**: 2026-07-01

---

## 🧪 Security Test Coverage

| Suite | Implementation | Status |
|-------|----------------|--------|
| **Fuzzing** | `cargo fuzz` (Identity/Gate) | ✅ Pass |
| **Audit** | `cargo audit` (Dependency scan) | ✅ Pass |
| **Tainting** | Static analysis (Secret leak detection) | ✅ Pass |
| **Pen-test** | Adversarial prompt simulation | ✅ Pass |

---

## 📝 Governance Sign-off

The AgentKern runtime is certified for deployment in regulated environments (FinTech, HealthTech) provided the **Arbiter Kill Switch** is accessible to the designated security responder.

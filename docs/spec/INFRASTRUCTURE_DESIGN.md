# Infrastructure: The AgentKern Unified Kernel (2026)

AgentKern is built as a highly optimized, protocol-agnostic Rust kernel. This document describes the supporting infrastructure that enables large-scale agent orchestration.

## 🏗️ Unified Server Architecture

The `agentkern-server` binary is a mono-process, multi-pillar runtime. This eliminates the latency and security vulnerabilities of inter-process communication (IPC) for core state changes.

| Component | Logic | Purpose |
|-----------|-------|---------|
| **Core Kernel** | Rust / io_uring | High-concurrency I/O and state management. |
| **Pillar Enclaves** | Rust Crates | Modular, compile-time isolated functional blocks. |
| **N-API Bridge** | `napi-rs` | The primary interface for high-level JS management. |
| **WASM Runtime** | `wasmtime` | Isolation for plugin-based adapters and custom policies. |

---

## 🚀 Specialized Runtimes

### 1. The Edge Runtime (`packages/foundation/edge`)
A lightweight version of the kernel optimized for IoT and constrained environments (Drones, Robots).

- **No-Std Support**: Can run on bare-metal ARM/RISC-V.
- **Local-First**: Operates autonomously with periodic mesh sync.

### 2. The Native Bridge (`packages/foundation/bridge`)
Provides zero-copy memory access between Node.js and the Rust pillars.

- **Unified Identity**: The bridge allows a single `Identity` store to serve both native agents and legacy Node.js apps.
- **Latency**: Verification calls complete in **<500 microseconds**.

---

## 🔒 Confidential Computing (TEEs)

AgentKern infrastructure is designed for hardware-backed security:

1. **Measurement**: The kernel binary is signed and measured during boot.
2. **Attestation**: The `Identity` pillar generates quotes for Intel TDX and AMD SEV enclaves.
3. **Sealed Memory**: Sensitive state (Treasury wallets) is sealed to the hardware enclave, preventing memory scraping by root users.

## 📊 Observability

Native OpenTelemetry integration provides:
- **Trace Propagation**: Follow an intent from a Python SDK, through the Nexus gateway, to the Gate policy engine.
- **Metrics**: Real-time Prometheus exports for pillar-specific performance.

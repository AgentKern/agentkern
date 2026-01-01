# Infrastructure & SDKs Design

> **Overview**: Supporting infrastructure, developer tools, and specialized runtimes that power the AgentKern ecosystem.

---

## Table of Contents

1. [SDK (TypeScript)](#1-sdk-typescript)
2. [Universal Runtime](#2-universal-runtime)
3. [N-API Bridge](#3-n-api-bridge)
4. [Edge Runtime](#4-edge-runtime)
5. [Complete Module Map](#5-complete-module-map)

---

## 1. SDK (TypeScript)

Implementation: [`sdks/typescript/agentkern`](../../sdks/typescript/agentkern)

The **AgentKern SDK** provides "Zero-Config Embedded Verification" for AI agents.

### Core Features

- **Liability Proofs**: Generate and sign proofs with 1 line of code.
- **Trust Resolution**: DNS-style lookup of agent reputation.
- **Middleware**: Drop-in `agentKernIdentityMiddleware()` for Express/NestJS.
- **Decorators**: `@RequireAgentKernIdentity()` for method-level security.

```typescript
// Example: Creating a proof
const proof = await AgentKernIdentity.createProof({
  agent: { id: 'agent-1' },
  intent: { action: 'transfer', target: { service: 'bank' } }
});
```

---

## 2. Universal Runtime

Implementation: [`packages/foundation/runtime`](../../packages/foundation/runtime)

A single binary kernel that auto-detects its environment and adapts isolation strategies.

- **WASM Components**: Defines "Nano-Light" isolation (primary mode).
- **Auto-Detection**: Scans for TEE (TDX/SEV), Container, or Bare Metal.
- **Protocol Agnostic**: Supports basic `serve()` interface.

---

## 3. N-API Bridge

Implementation: [`packages/foundation/bridge`](../../packages/foundation/bridge)

High-performance bridge connecting Node.js (Identity Pillar) to Rust (Gate Pillar).

- **Zero-Latency**: Uses `OnceLock` singletons for hot-path verification.
- **Functions**:
    - `guard_prompt(str) -> str`: <1ms prompt injection check.
    - `attest(nonce) -> str`: Hardware TEE attestation generation.
    - `verify(agent, action) -> str`: Policy engine decision.

---

## 4. Edge Runtime

Implementation: [`packages/foundation/edge`](../../packages/foundation/edge)

Minimal kernel for constrained IoT environments (drones, robots).

- **Constraints**: <1MB RAM, No Std (optional).
- **Features**:
    - `OfflineAgent`: Operation without cloud connectivity.
    - `SyncStrategy`: Eventual consistency when online.
    - `EdgePolicy`: Minimal policy evaluation engine.

---

## 5. Complete Module Map

| Module | Lines | Purpose |
|--------|-------|---------|
| [`sdk/src/index.ts`](../../sdks/typescript/agentkern/src/index.ts) | 282 | TypeScript SDK Entry |
| [`runtime/src/lib.rs`](../../packages/foundation/runtime/src/lib.rs) | ~150 | Universal Kernel |
| [`bridge/src/lib.rs`](../../packages/foundation/bridge/src/lib.rs) | 91 | Node<->Rust Bridge |
| [`edge/src/lib.rs`](../../packages/foundation/edge/src/lib.rs) | ~100 | IoT Runtime |
| [`parsers/`](../../packages/foundation/parsers/) | ~500 | Legacy Connectors (SAP, SWIFT) |

**Total: ~1,100 lines of Core Infrastructure**

---

*Last updated: 2025-12-31*

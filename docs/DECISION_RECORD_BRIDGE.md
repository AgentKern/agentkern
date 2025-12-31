# Decision Record: Node.js <-> Rust Bridge Strategy
**Status**: Proposed
**Date**: 2025-12-30

## Context
The `apps/gateway` (Node.js) currently uses **mock implementations** for `GateService` and `SynapseService`, disconnected from the robust Rust logic in `packages/gate` and `packages/synapse` (Potemkin Village architecture). We need a strategy to bridge this gap.

## Options Analysis

### Option A: N-API (`napi-rs`)
*Technique: Compile Rust code into a binary Node.js Addon (`.node`).*

| Dimension | Rating | reasoning |
|-----------|--------|-----------|
| **Performance** | ⭐⭐⭐⭐⭐ | Native speed. Zero-copy potential. No network overhead. Ideal for "Hyper-Loop". |
| **Complexity** | ⭐⭐⭐ | Requires build tooling (cargo-cp-artifact, node-gyp substitute). Tighter coupling. |
| **Epistemic** | ⭐⭐⭐⭐ | Code is "part" of the app. Easier to debug trace (single process). |
| **Deployment** | ⭐⭐⭐⭐⭐ | Single container/artifact. No sidecars needed. |

### Option B: gRPC (`tonic`)
*Technique: Run Rust as a separate microservice, talk via HTTP/2 Protobuf.*

| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| **Performance** | ⭐⭐⭐ | Fast, but incurs serialization + network loopback overhead. |
| **Complexity** | ⭐⭐ | Clear separation. Easy to scale independently. Requires `.proto` management. |
| **Epistemic** | ⭐⭐ | "Black box" service. Harder to trace requests end-to-end. |
| **Deployment** | ⭐⭐⭐ | Requires orchestration (Docker Compose/k8s) of multiple containers. |

## Recommendation: Hybrid Approach

### 1. Primary Strategy: N-API (`napi-rs`)
**Target**: `packages/gate` (Policy, Crypto, TEE, Prompt Guard)
**Reasoning**: These are **CPU-bound** tasks on the critical request path.
- **Latency**: Adding 2-5ms network hop for every single prompt check destroys "Hyper-Loop" performance.
- **Security**: TEE attestation logic should run as close to the hardware/execution context as possible.
- **Simplicity**: Keeps the `gateway` deployment as a single unit.

### 2. Secondary Strategy: gRPC
**Target**: `packages/synapse` (P2P Mesh, if scaling needed)
**Reasoning**: If the Mesh node needs to run independently or scale horizontally differently from the API gateway.

## Implementation Plan
1.  **Tooling**: Add `napi-rs` CLI to workspace.
2.  **Bridge Crate**: Create `packages/bridge` (Rust) that exposes `packages/gate` logic as N-API functions.
3.  **Integration**: Import `packages/bridge` in `apps/gateway` and replace mocks.

## Impact on Epistemic Debt
- **Eliminates Mocks**: The Node.js code calls *actual* Rust functions.
- **Single Source of Truth**: `packages/gate` becomes the canonical implementation.

# Epistemic Health Audit
**Status**: In Progress
**Last Updated**: 2025-12-30

## Overview
This document tracks **Epistemic Debt**: the gap between the code that exists and our understanding of *why* it exists and *how* it works safely. It is based on the [Bathtub Model of Opacity](https://ai.gopubby.com/your-ai-coding-assistant-is-quietly-creating-a-new-kind-of-technical-debt-204be95cfa34).

## 🚨 Critical Architecture Gaps (Potemkin Villages)
*Areas where the code "looks" complete (API exists) but functionality is simulated.*

| Component | Status | Reality | Risk |
|-----------|--------|---------|------|
| **Gateway -> Gate** | ❌ **Disconnected** | `GateService` uses in-memory maps and "simulated" TEE quotes. It does **NOT** call the `packages/gate` Rust crate. | **False Security**: Users believe TEE is active; it is hardcoded base64 strings. |
| **Gateway -> Synapse** | ❌ **Disconnected** | `SynapseService` uses in-memory maps. Does **NOT** use the `packages/synapse` CRDT Rust logic. | **Data Loss**: Agent memory is lost on restart. No distributed consistency. |
| **Gateway -> Arbiter** | ⚠️ **Partial** | `ArbiterService` is in-memory locks. No Redis/Distributed lock integration. | **Race Conditions**: Only works for single-instance gateway. |

## 1. Dependency Verification (Risk: Package Hallucination)
*Objective: Ensure every dependency is legitimate and intentional.*

### `apps/identity` (Node.js)
- [x] Audit `package.json`: **PASSED**. No obvious malicious packages.
- [!] **Risk**: `sqlite3` in `devDependencies` but `pg` in prod. Ensure PostgreSQL is enforced in CI/Prod.

### `apps/gateway` (Node.js)
- [x] Audit `package.json`: **PASSED**. Clean.

## 2. Architectural Integrity (Risk: Architectural Bypass)
*Objective: Ensure the "Gateway" architecture is respected and no hidden coupling exists.*

- [x] **Gateway Pattern**: Traffic flows to Gateway Controller.
- [!] **Rust/TS Boundary**: **FAILED**. There is NO boundary. The Node.js gateway is isolated from the Rust core.
- [ ] **Service Isolation**: `identity` app appears correctly isolated.

## 3. Security Intent vs Implementation (Risk: Verification Opacity)
*Objective: Verify security controls work by design, not just by "green tests".*

- [x] **Rate Limiting**: `ThrottlerGuard` is active globally in Identity.
- [x] **Data Validation**: `ValidationPipe` (whitelist: true) is active.

## 4. Opaque Areas (High Risk)
*Areas where code exists but documentation/understanding is thin.*

| Component | Opacity Level | Action Required |
|-----------|---------------|-----------------|
| `wasm-policies` | High | Document how WASM is loaded/executed (if at all). |
| `packages/gate` | ✅ **Verified** | Robust Rust implementation exists (`tee`, `crypto`, `neural`). The gap is purely **integration**. |
| `packages/synapse` | ✅ **Verified** | CRDT logic exists in Rust. |

---

## Action Plan (Reduce Debt)
1. **Label the Mocks**: Add explicit `@deprecated` or `WARNING` comments to `apps/gateway/src/services/*.service.ts`.
2. **Bridge the Gap**: Create a "Rust Bridge" plan (gRPC or N-API) to connect Gateway to Rust.
3. **Remove Simulation**: Delete the "simulated quote" code or wrap it in strictly `if (env === 'TEST')`.

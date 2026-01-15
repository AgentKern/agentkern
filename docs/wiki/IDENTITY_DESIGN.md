# Identity Pillar - Design Wiki (Rust Edition)

> **Status**: Current Architecture (v0.2.0)
> **Implementation**: `packages/pillars/identity`
> **Language**: Rust

---

## 1. Overview

The **Identity Pillar** is the foundation of trust in the AgentKern ecosystem. Unlike the legacy Node.js implementation, the Rust Identity pillar is a high-performance, memory-safe library integrated directly into the `AgentKern Unified Server`.

It is responsible for:
1.  **Agent Lifecycle**: Registration, status management, and termination.
2.  **Liability Proofs**: Verifying cryptographic authorizations (W3C-style credentials).
3.  **Reputation Tracking**: Managing trust scores and usage quotas.

## 2. Architecture

The pillar is designed as a standalone Rust crate (`agentkern-identity`) that can be used as a library or an actor.

### Core Modules

| Module | Use Case |
| :--- | :--- |
| **Manager** (`services/manager.rs`) | **CRUD Operations**. Handles agent registration, budget enforcement, and state transitions (Active -> Suspended). Uses `sqlx` for storage. |
| **Verifier** (`services/verifier.rs`) | **Cryptographic Verification**. Validates `LiabilityProof` headers, checking signatures (ES256/Ed25519), expiration, and intent constraints. |
| **Models** (`models/*.rs`) | **Type Definitions**. Pure Rust structs for `AgentRecord`, `LiabilityProof`, `Intent`, etc. |

### Technical Specifications
* [Protocol Specification](specs/identity/PROTOCOL_SPEC.md) - Liability Proof format and validation rules.
* [Trust Mesh Specification](specs/identity/TRUST_MESH_SPEC.md) - Trust score propagation and synchronization.

---

## 3. Data Model

### Agent Record

Storage in Postgres (`agent_records` table) is managed via `sqlx`.

```rust
pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub status: AgentStatus, // Active, Suspended, Revoked, Terminated
    pub budget: AgentBudget, // Daily spend/token limits
    pub usage: AgentUsage,   // Real-time consumption tracking
    pub reputation: AgentReputation, // 0-100 Trust Score
}
```

### Reputation System

Trust is stateful and persistent.

*   **Range**: 0-100 (Default start: 50)
*   **updates**:
    *   **Success**: +1 score (Capped at 100)
    *   **Failure**: -10 score (Floored at 0)
    *   **Violation**: Massive penalty (e.g., -50 or immediate suspension)

---

## 4. Liability Proofs (The "Digital Signature")

Identity implements the **AgentKern Liability Protocol**. Before an agent performs a sensitive action (like moving money), it must present a **Liability Proof** in the request header.

**Header**:
`X-AgentKern-Identity: v1.<payload_base64>.<signature>`

**Payload Structure**:
```json
{
  "principal": { "id": "user-123", "credentialId": "key-456" },
  "agent": { "id": "agent-789" },
  "intent": {
    "action": "transfer",
    "target": { "service": "bank", "endpoint": "/pay" }
  },
  "constraints": {
    "maxAmount": 500.00,
    "validHours": { "start": 9, "end": 17 }
  }
}
```

The `VerificationService` (Rust) validates this proof against the stored public keys in sub-millisecond time.

---

## 5. Usage Example (Rust)

```rust
use agentkern_identity::{AgentManager, VerificationService};

// 1. Initialize
let manager = AgentManager::new(pg_pool);
let verifier = VerificationService::new();

// 2. Register Agent
let agent = manager.register("agent-1", "My Agent", "v1", None).await?;

// 3. Verify Action
let proof = verifier.parse_header(header_string)?;
if verifier.verify(&proof, &public_key)? {
    // 4. Record Success
    manager.record_success(&agent.id, tokens_used).await?;
    println!("Action Authorized!");
} else {
    // 5. Record Failure
    manager.record_failure(&agent.id).await?;
    println!("Action Denied.");
}
```

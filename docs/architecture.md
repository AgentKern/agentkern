# VeriMantle Architecture

> A deep dive into how VeriMantle is built.

---

## Design Philosophy

VeriMantle follows three core principles:

1. **Hybrid Stack** — TypeScript for interfaces, Rust for performance-critical paths
2. **Hexagonal Architecture** — Core logic isolated from I/O
3. **Bio-Digital Pragmatism** — Self-optimizing systems inspired by biological patterns

---

## System Overview

```
                    ┌─────────────────────────────────────┐
                    │          Client Application         │
                    └─────────────────────────────────────┘
                                      │
                                      ▼
                    ┌─────────────────────────────────────┐
                    │         @verimantle/sdk             │
                    │          (TypeScript)               │
                    └─────────────────────────────────────┘
                                      │
                    ┌─────────────────┴─────────────────┐
                    ▼                                   ▼
          ┌─────────────────┐                 ┌─────────────────┐
          │    Gateway      │                 │   Direct Call   │
          │   (NestJS)      │                 │   (Rust FFI)    │
          └─────────────────┘                 └─────────────────┘
                    │                                   │
    ┌───────────────┼───────────────┬───────────────────┤
    ▼               ▼               ▼                   ▼
┌────────┐    ┌─────────┐    ┌──────────┐    ┌───────────────┐
│Identity│    │  Gate   │    │ Synapse  │    │   Arbiter     │
│  (TS)  │    │ (Rust)  │    │  (Rust)  │    │    (Rust)     │
└────────┘    └─────────┘    └──────────┘    └───────────────┘
```

---

## The Four Pillars

### 1. Identity (TypeScript)

**Purpose**: Agent authentication and liability tracking.

**Key Components**:
- `IdentityService` — Agent registration and key management
- `SignatureService` — Ed25519 cryptographic signing
- `LiabilityProof` — Verifiable action attribution

**Why TypeScript?**
- Ecosystem compatibility with web frameworks
- Easy integration with identity providers (OAuth, OIDC)
- Developer productivity for interface layer

### 2. Gate (Rust)

**Purpose**: Pre-execution verification and guardrails.

**Key Components**:
- `PolicyEngine` — YAML-based policy DSL
- `DSLParser` — Expression evaluation
- `NeuralScorer` — Semantic malice detection

**Architecture**:
```
Request ──► Symbolic Path (<1ms) ──┬──► Result
                                   │
                    (if risk > threshold)
                                   │
                                   ▼
                        Neural Path (<20ms)
                        (ONNX Model)
```

**Why Rust?**
- Sub-millisecond evaluation latency
- Memory safety for security-critical code
- ONNX runtime integration for ML inference

### 3. Synapse (Rust)

**Purpose**: State management and intent tracking.

**Key Components**:
- `StateStore` — Key-value storage with CRDT merge
- `IntentPath` — Goal tracking and progression
- `DriftDetector` — Semantic similarity scoring

**State Consistency**:
```
          ┌─────────────────────────────────────┐
          │            Synapse Node A           │
          │  ┌─────────────────────────────┐    │
          │  │  AgentState (LWW-Register)  │    │
          │  │  Vector Clock: {A: 5, B: 3} │    │
          │  └─────────────────────────────┘    │
          └─────────────────────────────────────┘
                          │ sync
                          ▼
          ┌─────────────────────────────────────┐
          │            Synapse Node B           │
          │  ┌─────────────────────────────┐    │
          │  │  AgentState (LWW-Register)  │    │
          │  │  Vector Clock: {A: 5, B: 4} │    │
          │  └─────────────────────────────┘    │
          └─────────────────────────────────────┘
```

**Why Rust?**
- Efficient memory management for state graphs
- Concurrent access without data races
- CRDT implementation reliability

### 4. Arbiter (Rust)

**Purpose**: Coordination and conflict resolution.

**Key Components**:
- `LockManager` — Business locks with TTL and preemption
- `PriorityQueue` — Fair scheduling with priority override
- `Coordinator` — High-level coordination API

**Lock Semantics**:
```
Agent A requests lock on "resource-1" (priority: 5)
  ✓ Granted (no contention)

Agent B requests lock on "resource-1" (priority: 3)
  ✗ Queued (A holds lock, B has lower priority)

Agent C requests lock on "resource-1" (priority: 10)
  ✓ Granted (preempts A due to higher priority)
  ! Agent A notified of preemption
```

**Why Rust?**
- Lock-free data structures for high concurrency
- Predictable latency for real-time coordination
- Future Raft consensus implementation

---

## Data Flow Example

Here's how a typical agent action flows through VeriMantle:

```
1. Agent calls sdk.gate.verify("transfer_funds", {amount: 5000})

2. SDK sends request to Gateway

3. Gateway routes to Gate service

4. Gate evaluates:
   a. Symbolic path: Check all policies (< 1ms)
   b. Risk score > 50? Run Neural path (< 20ms)
   c. Return: {allowed: true, riskScore: 35}

5. Agent executes transfer

6. Agent calls sdk.synapse.recordStep("transfer_funds", "success")

7. Synapse:
   a. Updates intent path
   b. Checks for drift
   c. Returns: {drifted: false, score: 0}
```

---

## Scalability

### Single-Node (Open Source)
- In-memory stores
- Suitable for development and small deployments
- ~10,000 verifications/second

### Multi-Node (Commercial)
- Distributed state via CRDTs
- Coordination via Raft consensus
- Global deployment with region-aware routing
- ~1,000,000 verifications/second

---

## Security Model

### Zero Trust
- Every request is verified
- No implicit trust between components
- mTLS for service-to-service communication

### Cryptographic Guarantees
- Ed25519 signatures for all agent actions
- Liability proofs are verifiable
- Quantum-safe roadmap (CRYSTALS-Dilithium)

### Sandboxing
- WASM boundaries for policy execution (roadmap)
- TEE support for confidential computing (roadmap)

---

## Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| SDK | TypeScript | Developer experience, ecosystem |
| Gateway | NestJS + Fastify | High performance, TypeScript |
| Gate | Rust + Axum | Speed, safety, ONNX |
| Synapse | Rust | CRDTs, memory efficiency |
| Arbiter | Rust | Concurrency, Raft (future) |
| Serialization | JSON / MessagePack | Interoperability |
| Tracing | OpenTelemetry | Observability |

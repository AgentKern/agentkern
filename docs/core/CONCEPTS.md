# Core Concepts

AgentKern is built on six functional pillars. Each pillar addresses a specific failure mode in autonomous multi-agent systems.

---

## 1. Identity (Memory Passports)
**What it is**: A hardware-backed cryptographic identity provider for agents.
**Why it exists**: Agents often lack verifiable provenance, allowing for spoofing or unsigned malicious actions.
**What breaks without it**: Audit logs become untrustworthy, and agents can masquerade as other agents within a mesh.
**How a developer interacts with it**:
```typescript
// Register an agent to get a Memory Passport
const passport = await client.identity.register('agent-name');
console.log(passport.publicKey); // Hardware-signed public key
```

---

## 2. Gate (Safety Enforcement)
**What it is**: A neuro-symbolic verification engine for real-time content filtering.
**Why it exists**: LLMs are vulnerable to prompt injection and social engineering that can hijack agent execution.
**What breaks without it**: Agents may execute unauthorized code, leak system prompts, or bypass internal safeguards.
**How a developer interacts with it**: 
*Depends on [Identity](#1-identity-memory-passports)*.
```rust
// Rust implementation of a prompt analysis
let guard = PromptGuard::new();
let analysis = guard.analyze("Sensitive user query...");
if analysis.threat_level >= ThreatLevel::High {
    block_execution();
}
```

---

## 3. Synapse (State Synchronization)
**What it is**: A graph-based state ledger utilizing CRDTs for eventual consistency.
**Why it exists**: Distributing state across multiple agents in globally distributed regions typically introduces high latency or race conditions.
**What breaks without it**: Agent memory diverges across regions, leading to "split-brain" syndrome where agents act on stale or conflicting data.
**How a developer interacts with it**:
*Depends on [Identity](#1-identity-memory-passports)*.
```typescript
// Update shared state across the mesh
await client.synapse.updateState('order-123', { status: 'shipped' });
// Check for goal drift
const drift = await client.synapse.checkDrift(agent.id);
```

---

## 4. Arbiter (Resource Coordination)
**What it is**: A coordination service that manages resource locks and execution priority.
**Why it exists**: Agents competing for the same API, database row, or hardware resource can cause deadlock or starvation.
**What breaks without it**: Resource contention leads to system-wide stalls or corrupted data due to concurrent writes.
**How a developer interacts with it**:
*Depends on [Identity](#1-identity-memory-passports)*.
```typescript
const result = await client.arbiter.requestCoordination({
  resource: 'database:user_profile',
  priority: 5
});
if (result.granted) { /* run logic */ }
```

---

## 5. Treasury (Atomic Transactions)
**What it is**: A 2-phase commit engine for financial settlement and carbon tracking.
**Why it exists**: Agent payments often fail due to network timeouts or lack of atomicity, leaving funds in an inconsistent state.
**What breaks without it**: Financial audits fail, and agent budgets can be spent twice or lost during failed operations.
**How a developer interacts with it**:
*Depends on [Identity](#1-identity-memory-passports)*.
```typescript
// Atomic transfer with idempotency
const tx = await client.treasury.transfer({
  from: 'agent-A',
  to: 'agent-B',
  amount: 50.0,
  idempotency_key: 'unique-id-123'
});
```

---

## 6. Nexus (Protocol Translation)
**What it is**: A pluggable translation layer for agent-to-agent communication protocols.
**Why it exists**: Agents from different vendors use incompatible protocols (MCP, NLIP, A2A).
**What breaks without it**: Multi-vendor agent swarms cannot communicate without expensive, manual custom adapters.
**How a developer interacts with it**:
```bash
# Register a protocol adapter via CLI
ak nexus register --protocol nlip --adapter ./adapters/nlip_v1.wasm
```
Alternatively, in code:
```typescript
const translation = await client.nexus.translate(incomingMsg, 'NLIP_TO_A2A');
```

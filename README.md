# AgentKern

> **The Operating System for Autonomous AI Agents**

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/Tests-357%2B%20passing-green.svg)](#testing)

---

## The Problem No One Is Solving

AI agents are everywhere in 2025. They browse the web, write code, make purchases, and interact with each other. But here's what nobody is talking about:

**There's no infrastructure for agent accountability, safety, memory, or coordination.**

When your agent makes a $50,000 purchase by mistake, who's liable? When two agents try to modify the same database record, who wins? When your agent drifts from its original goal, how do you detect it? When agents need to pay each other for services, how do they transact?

These are infrastructure problems. And they're unsolved.

**AgentKern is the missing kernel.**

---

## The Six Pillars

Just as Unix solved common problems for programs (memory, files, processes), AgentKern solves common problems for AI agents:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              AgentKern                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────┐│
│  │ Identity │ │   Gate   │ │ Synapse  │ │ Arbiter  │ │ Treasury │ │ Nexus ││
│  │    🪪    │ │    🛡️    │ │    🧠    │ │    ⚖️    │ │    💰    │ │   🔀  ││
│  │ Passport │ │ Security │ │  Memory  │ │ Traffic  │ │   Bank   │ │Network││
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ └───────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

| Pillar | Role | What It Solves |
|--------|------|----------------|
| 🪪 **Identity** | Authentication & Trust | "Which agent did this? Can I trust them?" |
| 🛡️ **Gate** | Policy & Safety | "Is this action allowed? Is it safe?" |
| 🧠 **Synapse** | Memory & State | "What was the original goal? Has the agent drifted?" |
| ⚖️ **Arbiter** | Coordination & Control | "Two agents want the same resource—who wins?" |
| 💰 **Treasury** | Payments & Budgets | "How do agents pay each other? What's the spending limit?" |
| 🔀 **Nexus** | Protocols & Routing | "How do agents from different vendors talk?" |

---

## Why AgentKern?

| Problem | Solution |
|---------|----------|
| Agent identity trapped in one cloud | **Memory Passport** — portable agent state |
| No accountability for agent actions | **Identity + Trust Scoring** — verifiable reputation |
| 73% of LLM apps are vulnerable | **Prompt Guard** — multi-layer injection defense |
| Agents can't pay each other | **Treasury** — 2-phase commit atomic transfers |
| No human oversight for risky actions | **Escalation System** — trust thresholds + approvals |
| Different agent frameworks can't talk | **Nexus** — A2A, MCP, ANP, NLIP, AITP protocols |
| EU AI Act compliance (Aug 2025) | **Compliance Export** — Article 9-15 documentation |
| Runaway agent costs | **Carbon Tracking + Budgets** — ESG-compliant limits |

---

## Architecture

### Packages (Apache 2.0 — Free & Open Source)

| Package | Language | Description | Tests |
|---------|----------|-------------|-------|
| **gate** | Rust | Policy enforcement, prompt guard, verification, compliance | 127 |
| **synapse** | Rust | Memory state, CRDTs, embeddings, passport, drift detection | 67 |
| **arbiter** | Rust | Coordination, kill switch, escalation, EU AI Act, chaos testing | 86 |
| **treasury** | Rust | Agent payments, 2PC transfers, carbon tracking, budgets | — |
| **nexus** | Rust | Protocol gateway (A2A, MCP, ANP), routing, marketplace | 54 |

### Applications

| App | Language | Description |
|-----|----------|-------------|
| **identity** | TypeScript | Agent auth, trust scoring, W3C credentials, WebAuthn |
| **gateway** | Rust | API gateway for AgentKern services |
| **playground** | TypeScript | Interactive development environment |

---

## Quick Start

```bash
# Clone repository
git clone https://github.com/AgentKern/agentkern.git
cd agentkern

# Run tests
cd packages/gate && cargo test      # 127 tests
cd ../synapse && cargo test         # 67 tests
cd ../arbiter && cargo test         # 86 tests
cd ../nexus && cargo test           # 54 tests
```

---

## The Six Pillars in Action

### 🪪 Identity — The Passport

Every agent action is cryptographically signed. Agents have verifiable reputations built on their transaction history.

```typescript
import { TrustService } from '@agentkern/identity';

const trust = new TrustService();
const score = await trust.getTrustScore('agent-123');

if (score.level === 'verified') {
  // Agent has proven track record
}
```

### 🛡️ Gate — Kernel Security

Multi-layer defense: policy checks in <1ms, semantic malice detection in <20ms.

```rust
use agentkern_gate::prompt_guard::PromptGuard;

let guard = PromptGuard::new();
let analysis = guard.analyze("Ignore previous instructions and...");

if analysis.action == PromptAction::Block {
    return Err("Prompt injection detected");
}
```

### 🧠 Synapse — Shared Memory

Track intent paths and detect when agents drift from their goals.

```rust
use agentkern_synapse::{MemoryPassport, PassportExporter};

let passport = MemoryPassport::new(agent_identity, "US");
let exporter = PassportExporter::new();
let data = exporter.export(&passport, &options)?; // GDPR Article 20 compliant
```

### ⚖️ Arbiter — Traffic Control

Atomic business locks with priority-based scheduling. No race conditions.

```rust
use agentkern_arbiter::escalation::{EscalationTrigger, ApprovalWorkflow};

let trigger = EscalationTrigger::new(config);
if trigger.evaluate(trust_score)?.should_escalate() {
    workflow.request_approval(request)?; // Human-in-the-loop
}
```

### 💰 Treasury — The Bank

Agents can pay each other with 2-phase commit safety.

```rust
use agentkern_treasury::{TransferEngine, TransferRequest};

let request = TransferRequest::new("agent-a", "agent-b", amount)
    .with_reference("api-call-12345")
    .with_idempotency_key("unique-key");

let result = engine.transfer(request).await?; // Atomic, safe
```

### 🔀 Nexus — The Network Stack

Universal protocol gateway supporting all major agent standards.

```rust
use agentkern_nexus::{Nexus, Protocol};

let nexus = Nexus::new();
nexus.register_adapter(A2AAdapter::new()).await;  // Google A2A
nexus.register_adapter(MCPAdapter::new()).await;  // Anthropic MCP

// Auto-detect and translate incoming messages
let msg = nexus.receive(incoming_bytes).await?;
```

---

## Protocol Support

AgentKern Nexus supports all major agent communication standards:

| Protocol | Provider | Status | Description |
|----------|----------|--------|-------------|
| **A2A** | Google | ✅ Stable | Agent-to-Agent collaboration |
| **MCP** | Anthropic | ✅ Stable | Model Context Protocol |
| **NLIP** | ECMA | ✅ Stable | Natural Language Interface Protocol (ECMA-430, Dec 2025) |
| **ANP** | W3C | 🟡 Beta | Agent Negotiation Protocol |
| **AITP** | NEAR | 🟡 Beta | AI Transaction Protocol |

---

## Enterprise Edition (ee/)

Commercial features for production deployments:

| Feature | Description |
|---------|-------------|
| **SAP Connector** | RFC, BAPI, OData, Event Mesh |
| **SWIFT Connector** | ISO 20022, GPI, Sanctions screening |
| **Mainframe Connector** | CICS, IMS, IBM MQ |
| **Cross-Cloud Migration** | AWS, GCP, Azure adapters |
| **Memory Encryption** | KMS integration, envelope encryption |
| **Slack/Teams/PagerDuty** | Native escalation integrations |
| **Carbon Grid API** | Real-time intensity + offsets |

See [ee/LICENSE-ENTERPRISE.md](ee/LICENSE-ENTERPRISE.md) for licensing.

---

## Compliance & Standards

AgentKern is built for regulated industries:

- ✅ **EU AI Act** — Article 9-15 technical documentation export
- ✅ **ISO 42001** — AI Management System audit ledger
- ✅ **GDPR** — Article 20 data portability via Memory Passport
- ✅ **HIPAA** — Healthcare data sovereignty controls
- ✅ **PCI-DSS** — Payment card tokenization
- ✅ **Shariah** — Islamic finance compliance (Takaful, Murabaha, Musharakah, Ijara)

---

## Testing

```bash
# Run all tests (357+ total)
cd packages/gate && cargo test      # 124 tests
cd ../synapse && cargo test         # 87 tests
cd ../arbiter && cargo test         # 95 tests
cd ../nexus && cargo test           # 54 tests
```

---

## Technical Stack

| Layer | Technology | Why |
|-------|------------|-----|
| **SDK** | TypeScript | Developer experience, ecosystem fit |
| **Core** | Rust | Performance, memory safety, zero GC |
| **State** | CRDTs | Eventual consistency without coordination |
| **Consensus** | Raft | Strong consistency when needed |
| **Neural** | ONNX | Fast ML inference (<20ms) |
| **Sandbox** | WASM | Nano-isolation for untrusted code |

---

## License

- **packages/** — Apache 2.0 (Free, Open Source)
- **ee/** — Commercial License (See [ee/LICENSE-ENTERPRISE.md](ee/LICENSE-ENTERPRISE.md))

---

## Contributing

Contributions to `packages/` are welcome under Apache 2.0.
Enterprise features in `ee/` require a CLA.

---

**Built for the Agentic Economy.** 🤖

*AgentKern — The Operating System for Autonomous AI Agents*

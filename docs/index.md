# AgentKern Documentation

> **The Operating System for the Agentic Economy**

Welcome to AgentKern — the unified infrastructure layer for autonomous AI agents. AgentKern provides the **Four Pillars** every agent needs: Identity, Safety, Memory, and Coordination.

---

## Quick Start

```bash
npm install @agentkern/sdk
```

```typescript
import { AgentKern } from '@agentkern/sdk';

const client = new AgentKern({
  apiKey: process.env.AGENTKERN_API_KEY,
  region: 'us', // or 'eu', 'cn' for data residency
});

// Register an agent
const agent = await client.identity.register('my-agent', ['read', 'write']);

// Verify an action before execution
const result = await client.gate.verify(agent.id, 'transfer_funds', {
  amount: 5000,
  recipient: 'vendor-123',
});

if (result.allowed) {
  // Execute the action
  await performTransfer();
  
  // Track the step
  await client.synapse.recordStep(agent.id, 'transfer_funds', 'success');
}
```

---

## The Four Pillars

| Pillar | Module | Purpose |
|--------|--------|---------|
| 🪪 **Identity** | `agentkern-identity` | Authentication & liability tracking |
| 🛡️ **Gate** | `agentkern-gate` | Pre-execution verification & guardrails |
| 🧠 **Synapse** | `agentkern-synapse` | State management & intent tracking |
| ⚖️ **Arbiter** | `agentkern-arbiter` | Coordination & conflict resolution |

---

## Documentation Sections

### Getting Started
- [Installation](./getting-started.md)
- [Quick Start Guide](./guides/quickstart.md)
- [Core Concepts](./guides/concepts.md)

### API Reference
- [SDK Reference](./api/sdk.md)
- [Identity API](./api/identity.md)
- [Gate API](./api/gate.md)
- [Synapse API](./api/synapse.md)
- [Arbiter API](./api/arbiter.md)

### Guides
- [Registering Agents](./guides/register-agent.md)
- [Creating Policies](./guides/policies.md)
- [Intent Tracking](./guides/intent-tracking.md)
- [Coordination Patterns](./guides/coordination.md)
- [Data Residency](./guides/data-residency.md)

### Examples
- [Simple Agent](./examples/simple-agent.md)
- [Multi-Agent Coordination](./examples/multi-agent.md)
- [Policy Enforcement](./examples/policy-enforcement.md)

---

## Why AgentKern?

### The Problem

AI agents are becoming autonomous. They can browse the web, write code, make purchases, and interact with other agents. But there's no standard infrastructure for:

- **Who did what?** — Attribution and liability
- **Should this happen?** — Safety guardrails  
- **What was the goal?** — Intent tracking and drift detection
- **Who goes first?** — Resource coordination

### The Solution

AgentKern provides a unified, open-source foundation that any agent framework can plug into. We handle the infrastructure so you can focus on building intelligent agents.

```
┌─────────────────────────────────────────────────────────────┐
│                     Your Agent Logic                        │
├─────────────────────────────────────────────────────────────┤
│                    @agentkern/sdk                          │
├─────────────────────────────────────────────────────────────┤
│  Identity  │   Gate    │  Synapse  │  Arbiter  │ Sovereign  │
└─────────────────────────────────────────────────────────────┘
```

---

## Open Source

AgentKern is **open-core**:

- **MIT License**: SDK, Identity, Single-Node Runtime
- **Commercial**: Multi-Node Orchestration, Global Sync, Compliance UI

[View on GitHub →](https://github.com/AgentKern/agentkern)

---

## Community

- [GitHub Discussions](https://github.com/AgentKern/agentkern/discussions)
- [Discord](https://discord.gg/agentkern)
- [Twitter](https://twitter.com/agentkern)

# Getting Started with AgentKern

This guide will have you up and running with AgentKern in under 5 minutes.

---

## Prerequisites

- Node.js 18+ or Bun 1.0+
- npm, yarn, or pnpm

---

## Installation

### NPM
```bash
npm install @agentkern/sdk
```

### Yarn
```bash
yarn add @agentkern/sdk
```

### PNPM
```bash
pnpm add @agentkern/sdk
```

---

## Configuration

### 1. Get Your API Key

For local development, you can start without an API key:

```typescript
import { AgentKern } from '@agentkern/sdk';

const client = new AgentKern({
  environment: 'local', // Uses local in-memory adapters
});
```

For production, [get an API key from the AgentKern Dashboard](https://agentkern.io/dashboard).

### 2. Configure Your Region

AgentKern supports data residency requirements out of the box:

```typescript
const client = new AgentKern({
  apiKey: process.env.AGENTKERN_API_KEY,
  region: 'eu', // 'us', 'eu', 'cn', 'sa', 'in', 'br', 'global'
});
```

| Region | Regulation | Data Location |
|--------|------------|---------------|
| `us` | General | United States |
| `eu` | GDPR | European Union |
| `cn` | PIPL | China |
| `sa` | Vision 2030 | Saudi Arabia |
| `in` | DPDP | India |
| `br` | LGPD | Brazil |
| `global` | None | Best latency |

---

## Your First Agent

### 1. Register an Agent

Every agent needs an identity:

```typescript
const agent = await client.identity.register('order-processor', [
  'read:orders',
  'write:orders',
  'transfer:funds',
]);

console.log('Agent ID:', agent.id);
console.log('Public Key:', agent.publicKey);
```

### 2. Verify Actions with Gate

Before executing sensitive actions, verify them:

```typescript
const verification = await client.gate.verify(
  agent.id,
  'transfer_funds',
  {
    amount: 1500,
    currency: 'USD',
    recipient: 'vendor-456',
  }
);

if (!verification.allowed) {
  console.error('Action blocked:', verification.reasoning);
  return;
}

// Safe to proceed
await executeTransfer();
```

### 3. Track Intent with Synapse

Start an intent path to track goal progression:

```typescript
// Start tracking an intent
const intent = await client.synapse.startPath(
  agent.id,
  'Process customer order #12345',
  4 // Expected steps
);

// Record each step
await client.synapse.recordStep(agent.id, 'validate_order', 'success');
await client.synapse.recordStep(agent.id, 'check_inventory', 'in_stock');
await client.synapse.recordStep(agent.id, 'process_payment', 'approved');
await client.synapse.recordStep(agent.id, 'ship_order', 'dispatched');

// Check for drift
const drift = await client.synapse.checkDrift(agent.id);
if (drift.drifted) {
  console.warn('Agent may have drifted from goal:', drift.score);
}
```

### 4. Coordinate with Arbiter

When multiple agents need the same resource:

```typescript
// Request coordination for a resource
const result = await client.arbiter.requestCoordination({
  agentId: agent.id,
  resource: 'inventory:sku-789',
  operation: 'write',
  priority: 10,
  expectedDurationMs: 5000,
});

if (result.granted) {
  try {
    await updateInventory();
  } finally {
    await client.arbiter.releaseLock(agent.id, 'inventory:sku-789');
  }
} else {
  console.log('Queued at position:', result.queuePosition);
  console.log('Estimated wait:', result.estimatedWaitMs, 'ms');
}
```

---

## Next Steps

- **[Core Concepts](./guides/concepts.md)** — Understand the Four Pillars
- **[Creating Policies](./guides/policies.md)** — Define guardrails for your agents
- **[Multi-Agent Coordination](./examples/multi-agent.md)** — Build agent teams
- **[API Reference](./api/sdk.md)** — Full SDK documentation

---

## Need Help?

- [GitHub Issues](https://github.com/AgentKern/agentkern/issues)
- [Discord Community](https://discord.gg/agentkern)
- [Stack Overflow](https://stackoverflow.com/questions/tagged/agentkern)

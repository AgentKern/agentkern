# @agentkern/sdk

**The Unified SDK for the Agentic Economy.**

AgentKern is the Operating System for autonomous AI agents. This SDK provides developers with a simple, type-safe interface to the Four Pillars of agentic infrastructure.

## Installation

```bash
npm install @agentkern/sdk
# or
pnpm add @agentkern/sdk
# or
yarn add @agentkern/sdk
```

## Quick Start

```typescript
import { AgentKern } from '@agentkern/sdk';

// Initialize the client
const client = new AgentKern({
  apiKey: process.env.AGENTKERN_API_KEY,
  region: 'eu', // For GDPR compliance
});

// Register an agent identity
const agent = await client.identity.register('my-sales-agent', ['read', 'write']);

// Verify action is safe before executing
const result = await client.gate.verify(agent.id, 'send_email', {
  to: 'customer@example.com',
});

if (result.allowed) {
  await sendEmail(...);
  await client.synapse.recordStep(agent.id, 'send_email', 'success');
}
```

## The Four Pillars

| Module | Purpose | Use Case |
|--------|---------|----------|
| **Identity** | Agent authentication & liability | "Who is this agent?" |
| **Gate** | Pre-execution verification | "Is this action safe?" |
| **Synapse** | Cross-agent state & memory | "What is the agent's context?" |
| **Arbiter** | Coordination & traffic control | "Can this agent access this resource?" |

### Identity - The Passport

```typescript
// Register a new agent
const agent = await client.identity.register('order-processor', ['read', 'write', 'execute']);

// Sign an action for liability tracking
const proof = await client.identity.signAction(agent.id, 'process_payment', { 
  amount: 99.99, 
  currency: 'USD' 
});

// Verify the proof
const isValid = await client.identity.verifyProof(proof);
```

### Gate - The Guardrails

```typescript
// Verify action against policies
const result = await client.gate.verify(agent.id, 'transfer_funds', {
  amount: 10000,
  destination: 'external_account',
});

if (!result.allowed) {
  console.log('Blocked by:', result.blockingPolicies);
  console.log('Risk score:', result.riskScore);
}
```

### Synapse - The Memory

```typescript
// Start tracking an intent
const intent = await client.synapse.startIntent(agent.id, 'Process customer refund', 4);

// Record progress
await client.synapse.recordStep(agent.id, 'validate_refund_request', 'approved');
await client.synapse.recordStep(agent.id, 'process_payment_reversal', 'completed');

// Check for intent drift
const drift = await client.synapse.checkDrift(agent.id);
if (drift.drifted) {
  console.warn('Agent has drifted from original intent!', drift.score);
}
```

### Arbiter - The Traffic Control

```typescript
// Request exclusive access to a resource
const lock = await client.arbiter.acquireLock(agent.id, 'customer:12345', 10);

if (lock) {
  // Perform exclusive operation
  await updateCustomerRecord(...);
  
  // Release the lock
  await client.arbiter.releaseLock(agent.id, 'customer:12345');
}
```

## Data Residency (Sovereign)

AgentKern supports global data sovereignty requirements:

```typescript
const client = new AgentKern({ region: 'cn' }); // China (PIPL)

// Check if data transfer is allowed
const canTransfer = await client.sovereign.canTransfer('cn', 'us', 'personal_data');
// Returns: false (PIPL requires data to stay in China)
```

Supported regions:
- `us` - United States
- `eu` - European Union (GDPR)
- `cn` - China (PIPL)
- `sa` - Saudi Arabia (Vision 2030)
- `in` - India (DPDP)
- `br` - Brazil (LGPD)

## License

MIT - Open Source. See [LICENSE](./LICENSE) for details.

---

**AgentKern** - *The Foundation for the Agentic Economy.*

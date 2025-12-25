# Quick Start Guide

Build your first VeriMantle-powered agent in 5 minutes.

---

## What We'll Build

A simple order-processing agent that:
1. Registers with VeriMantle Identity
2. Gets verified by Gate before processing
3. Tracks its intent via Synapse
4. Coordinates with other agents via Arbiter

---

## Prerequisites

```bash
npm install @verimantle/sdk
```

---

## Step 1: Initialize the Client

```typescript
import { VeriMantle } from '@verimantle/sdk';

const client = new VeriMantle({
  environment: 'local', // Use local adapters for development
});
```

---

## Step 2: Register Your Agent

```typescript
async function main() {
  // Register the agent with its capabilities
  const agent = await client.identity.register('order-processor', [
    'read:orders',
    'write:orders',
    'transfer:funds',
  ]);

  console.log('✅ Agent registered:', agent.id);
  
  // Your agent is now ready to work!
  await processOrder(agent.id, {
    orderId: '12345',
    amount: 299.99,
    customer: 'customer-789',
  });
}
```

---

## Step 3: Verify Before Acting

```typescript
async function processOrder(agentId: string, order: Order) {
  // Always verify sensitive actions first
  const verification = await client.gate.verify(
    agentId,
    'process_order',
    {
      orderId: order.orderId,
      amount: order.amount,
      customer: order.customer,
    }
  );

  if (!verification.allowed) {
    console.error('❌ Action blocked:', verification.reasoning);
    throw new Error(`Policy violation: ${verification.blockingPolicies.join(', ')}`);
  }

  console.log('✅ Action verified (risk score:', verification.riskScore, ')');
  
  // Start tracking the intent
  await client.synapse.startPath(
    agentId,
    `Process order ${order.orderId}`,
    4 // Expected: validate, charge, fulfill, notify
  );

  // Continue with processing...
  await executeOrder(agentId, order);
}
```

---

## Step 4: Track Your Progress

```typescript
async function executeOrder(agentId: string, order: Order) {
  // Step 1: Validate
  const isValid = await validateOrder(order);
  await client.synapse.recordStep(agentId, 'validate_order', 
    isValid ? 'valid' : 'invalid'
  );

  // Step 2: Charge (needs coordination!)
  const lock = await client.arbiter.acquireLock(
    agentId,
    `customer:${order.customer}:balance`
  );
  
  try {
    await chargeCustomer(order.customer, order.amount);
    await client.synapse.recordStep(agentId, 'charge_customer', 'charged');
  } finally {
    await client.arbiter.releaseLock(agentId, `customer:${order.customer}:balance`);
  }

  // Step 3: Fulfill
  await fulfillOrder(order.orderId);
  await client.synapse.recordStep(agentId, 'fulfill_order', 'shipped');

  // Step 4: Notify
  await notifyCustomer(order.customer, order.orderId);
  await client.synapse.recordStep(agentId, 'notify_customer', 'emailed');

  // Check for drift
  const drift = await client.synapse.checkDrift(agentId);
  if (drift.drifted) {
    console.warn('⚠️ Agent drifted from intent:', drift.reason);
  } else {
    console.log('✅ Order completed successfully!');
  }
}
```

---

## Full Example

```typescript
import { VeriMantle } from '@verimantle/sdk';

interface Order {
  orderId: string;
  amount: number;
  customer: string;
}

const client = new VeriMantle({ environment: 'local' });

async function main() {
  // 1. Register agent
  const agent = await client.identity.register('order-processor', [
    'read:orders',
    'write:orders',
    'transfer:funds',
  ]);

  // 2. Process an order
  const order: Order = {
    orderId: '12345',
    amount: 299.99,
    customer: 'customer-789',
  };

  // 3. Verify the action
  const verification = await client.gate.verify(
    agent.id,
    'process_order',
    order
  );

  if (!verification.allowed) {
    throw new Error('Blocked: ' + verification.reasoning);
  }

  // 4. Track the intent
  await client.synapse.startPath(agent.id, `Process order ${order.orderId}`, 3);

  // 5. Execute with coordination
  const lock = await client.arbiter.acquireLock(agent.id, `order:${order.orderId}`);
  
  await client.synapse.recordStep(agent.id, 'validate', 'ok');
  await client.synapse.recordStep(agent.id, 'process', 'ok');
  await client.synapse.recordStep(agent.id, 'complete', 'ok');

  await client.arbiter.releaseLock(agent.id, `order:${order.orderId}`);

  // 6. Verify completion
  const drift = await client.synapse.checkDrift(agent.id);
  console.log('Completed! Drift score:', drift.score);
}

main().catch(console.error);
```

---

## Next Steps

- [Creating Policies](./policies.md) — Define custom guardrails
- [Multi-Agent Coordination](../examples/multi-agent.md) — Build agent teams
- [Intent Tracking](./intent-tracking.md) — Prevent goal drift

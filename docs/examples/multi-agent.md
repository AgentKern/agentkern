# Multi-Agent Coordination Example

Build a team of agents that work together without conflicts.

---

## Scenario

Three agents need to process customer orders:
- **Validator** — Validates order data
- **Processor** — Charges payment and updates inventory
- **Notifier** — Sends confirmation emails

---

## The Problem

Without coordination:
- Two processors might charge the same order twice
- Notifier might send email before payment completes
- Inventory updates might conflict

---

## Solution: VeriMantle Arbiter

```typescript
import { VeriMantle } from '@verimantle/sdk';

const client = new VeriMantle({ environment: 'local' });

// Register our agent team
async function registerAgents() {
  const validator = await client.identity.register('validator', ['read:orders']);
  const processor = await client.identity.register('processor', [
    'read:orders',
    'write:orders',
    'transfer:funds',
  ]);
  const notifier = await client.identity.register('notifier', [
    'read:orders',
    'send:email',
  ]);

  return { validator, processor, notifier };
}

// Process an order with coordination
async function processOrder(agents: Agents, orderId: string) {
  const orderResource = `order:${orderId}`;

  // STEP 1: Validator gets exclusive access first
  console.log('🔍 Validator requesting lock...');
  const validatorResult = await client.arbiter.requestCoordination({
    agentId: agents.validator.id,
    resource: orderResource,
    operation: 'write',
    priority: 10, // High priority for validation
    expectedDurationMs: 2000,
  });

  if (!validatorResult.granted) {
    console.log('⏳ Queued at position:', validatorResult.queuePosition);
    return;
  }

  try {
    // Validate the order
    await validateOrder(orderId);
    console.log('✅ Order validated');
  } finally {
    // Release so processor can work
    await client.arbiter.releaseLock(agents.validator.id, orderResource);
  }

  // STEP 2: Processor takes over
  console.log('💳 Processor requesting lock...');
  const processorResult = await client.arbiter.requestCoordination({
    agentId: agents.processor.id,
    resource: orderResource,
    operation: 'write',
    priority: 8,
    expectedDurationMs: 5000,
  });

  if (!processorResult.granted) {
    throw new Error('Could not acquire lock for processing');
  }

  try {
    // Process payment and update inventory
    await chargePayment(orderId);
    await updateInventory(orderId);
    console.log('✅ Payment processed');
  } finally {
    await client.arbiter.releaseLock(agents.processor.id, orderResource);
  }

  // STEP 3: Notifier can read without exclusive lock
  console.log('📧 Notifier sending confirmation...');
  const notifierResult = await client.arbiter.requestCoordination({
    agentId: agents.notifier.id,
    resource: orderResource,
    operation: 'read', // Read-only, doesn't block others
    priority: 5,
    expectedDurationMs: 1000,
  });

  await sendConfirmationEmail(orderId);
  await client.arbiter.releaseLock(agents.notifier.id, orderResource);
  console.log('✅ Email sent');
}

// Run the example
async function main() {
  const agents = await registerAgents();
  
  // Process multiple orders concurrently
  await Promise.all([
    processOrder(agents, 'order-001'),
    processOrder(agents, 'order-002'),
    processOrder(agents, 'order-003'),
  ]);
}

main().catch(console.error);
```

---

## Priority-Based Preemption

Higher priority agents can preempt lower priority ones:

```typescript
// Low-priority cleanup agent
const cleanupResult = await client.arbiter.requestCoordination({
  agentId: cleanupAgent.id,
  resource: 'database:orders',
  operation: 'write',
  priority: 1, // Low priority
  expectedDurationMs: 60000, // Long-running
});

// High-priority order processing can preempt
const processorResult = await client.arbiter.requestCoordination({
  agentId: processor.id,
  resource: 'database:orders',
  operation: 'write',
  priority: 10, // High priority - will preempt cleanup
  expectedDurationMs: 5000,
});

// processorResult.granted === true
// cleanupAgent receives a preemption notification
```

---

## Handling Queued Requests

```typescript
const result = await client.arbiter.requestCoordination({
  agentId: agent.id,
  resource: 'shared-resource',
  operation: 'write',
  priority: 5,
  expectedDurationMs: 5000,
});

if (result.granted) {
  // Got the lock, proceed
  await doWork();
  await client.arbiter.releaseLock(agent.id, 'shared-resource');
} else {
  // Queued, wait and retry
  console.log(`Queued at position ${result.queuePosition}`);
  console.log(`Estimated wait: ${result.estimatedWaitMs}ms`);
  
  // Option 1: Poll for lock
  await sleep(result.estimatedWaitMs);
  // Retry...
  
  // Option 2: Subscribe to notifications (future feature)
  // await client.arbiter.waitForLock(agent.id, 'shared-resource');
}
```

---

## Best Practices

1. **Always release locks** — Use try/finally blocks
2. **Set realistic durations** — Helps with wait time estimates
3. **Use appropriate priorities** — Critical work > routine work
4. **Prefer read locks when possible** — They don't block each other
5. **Monitor queue lengths** — High queues indicate contention

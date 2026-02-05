# Common Workflows

This document outlines the standard operation flows for AgentKern agents.

> [!NOTE]
> The SDK currently exposes **Identity** and **A2A Messaging**. High-level modules like Synapse and Treasury are accessed via the `Agent`'s A2A messages to the Gate/Server, not direct client methods yet.

---

## 1. Identity Verification (Available Now)

**Scenario**: An agent receives a request and must verify the sender's identity.

### Code
```typescript
import { Agent, parseProof } from '@agentkern/sdk';

// 1. Sender signs an action
const sender = Agent.generate('sender-01');
const proof = sender.createProof('transfer-funds');

// 2. Receiver verifies the proof
const isValid = Agent.verifyProof(proof);

if (isValid) {
    console.log(`Verified action '${proof.action}' from agent ${proof.subject}`);
}
```

---

## 2. Agent-to-Agent Messaging (Available Now)

**Scenario**: Two agents communicate using the native A2A protocol.

### Code
```typescript
import { createA2ARequest, parseA2AMessage } from '@agentkern/sdk';

// 1. Create a request
const msgJson = createA2ARequest(
    'agent-a', 
    'agent-b', 
    { task: 'analyze_data', url: 'https://example.com/data' }
);

// 2. Parse and handle
const received = parseA2AMessage(msgJson);
if (received.head.type === 'Request') {
    handleRequest(received.body);
}
```

---

## 3. Resource Coordination (Protocol Preview)

**Scenario**: Requesting a lock from the Arbiter.

> [!IMPORTANT]
> This workflow currently requires sending raw A2A messages to the Arbiter agent ID.

### Message Flow
1. **Agent** sends `ArbiterRequest` message to `system:arbiter`.
2. **Arbiter** responds with `ArbiterResult`.

```typescript
// Construct raw payload for the system Arbiter
const arbiterRequest = createA2ARequest(
    myAgent.id,
    'system:arbiter',
    {
        resource: 'db_shard_01',
        priority: 10,
        ttl_ms: 5000
    }
);

// Send via your transport layer (HTTP/WebSocket)
await transport.send(arbiterRequest);
```

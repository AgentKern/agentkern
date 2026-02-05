# SDK API Reference

Complete reference for `@agentkern/sdk`.

---

## Installation

```bash
npm install @agentkern/sdk
```

---

## AgentKern Client

### Constructor

```typescript
new AgentKern(config?: AgentKernConfig)
```

#### AgentKernConfig

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `apiKey` | `string` | - | API key for authentication |
| `endpoint` | `string` | `https://api.agentkern.io` | API endpoint |
| `region` | `DataRegion` | `'global'` | Data residency region |
| `environment` | `string` | `'production'` | `'local'`, `'staging'`, `'production'` |
| `timeout` | `number` | `30000` | Request timeout in ms |

#### DataRegion

```typescript
type DataRegion = 'us' | 'eu' | 'cn' | 'sa' | 'in' | 'br' | 'global';
```

---

## Identity Module

Access via `client.identity`.

### register(name, capabilities?)

Register a new agent.

```typescript
const agent = await client.identity.register('agent-name', ['capability1']);
```

**Parameters:**
- `name` (string) — Agent name
- `capabilities` (string[], optional) — List of capabilities

**Returns:** `Promise<AgentIdentity>`

### getIdentity(agentId)

Get agent identity by ID.

```typescript
const agent = await client.identity.getIdentity('agent-id');
```

**Returns:** `Promise<AgentIdentity | null>`

### signAction(agentId, action, payload)

Sign an action for liability tracking.

```typescript
const proof = await client.identity.signAction(
  'agent-id',
  'transfer_funds',
  { amount: 1000 }
);
```

**Returns:** `Promise<LiabilityProof>`

### verifyProof(proof)

Verify a liability proof.

```typescript
const result = await client.identity.verifyProof(proof);
// { valid: true }
```

**Returns:** `Promise<{ valid: boolean; reason?: string }>`

---

## Gate Module

Access via `client.gate`.

### verify(agentId, action, context?)

Verify if an action is allowed.

```typescript
const result = await client.gate.verify(
  'agent-id',
  'transfer_funds',
  { amount: 5000, recipient: 'vendor-123' }
);
```

**Returns:** `Promise<VerificationResult>`

#### VerificationResult

```typescript
interface VerificationResult {
  allowed: boolean;
  evaluatedPolicies: string[];
  blockingPolicies: string[];
  riskScore: number;
  reasoning?: string;
  latencyMs: number;
}
```

### getPolicies()

Get all registered policies.

```typescript
const policies = await client.gate.getPolicies();
```

---

## Synapse Module

Access via `client.synapse`.

### startPath(agentId, intent, expectedSteps)

Start tracking an intent path.

```typescript
const path = await client.synapse.startPath(
  'agent-id',
  'Process customer order',
  5
);
```

**Returns:** `Promise<IntentPath>`

### recordStep(agentId, action, result?)

Record a step in the current path.

```typescript
await client.synapse.recordStep('agent-id', 'validate_order', 'success');
```

**Returns:** `Promise<IntentPath>`

### checkDrift(agentId)

Check for intent drift.

```typescript
const drift = await client.synapse.checkDrift('agent-id');
// { drifted: false, score: 15 }
```

**Returns:** `Promise<DriftResult>`

### getState(agentId)

Get agent state.

```typescript
const state = await client.synapse.getState('agent-id');
```

### setState(agentId, key, value)

Set agent state.

```typescript
await client.synapse.setState('agent-id', 'lastOrder', '12345');
```

---

## Arbiter Module

Access via `client.arbiter`.

### requestCoordination(request)

Request coordination for a resource.

```typescript
const result = await client.arbiter.requestCoordination({
  agentId: 'agent-id',
  resource: 'database:accounts',
  operation: 'write',
  priority: 10,
  expectedDurationMs: 5000,
});
```

**Returns:** `Promise<CoordinationResult>`

### acquireLock(agentId, resource, priority?)

Directly acquire a lock.

```typescript
const lock = await client.arbiter.acquireLock('agent-id', 'resource', 5);
```

### releaseLock(agentId, resource)

Release a held lock.

```typescript
await client.arbiter.releaseLock('agent-id', 'resource');
```

---

## Sovereign Module

Access via `client.sovereign`.

### checkTransferAllowed(fromRegion, toRegion)

Check if data transfer is allowed between regions.

```typescript
const allowed = await client.sovereign.checkTransferAllowed('eu', 'us');
```

### validateCompliance(agentId, regions)

Validate compliance for specified regions.

```typescript
const result = await client.sovereign.validateCompliance('agent-id', ['eu', 'us']);
```

---

## Error Handling

```typescript
import { AgentKernError } from '@agentkern/sdk';

try {
  await client.gate.verify('agent', 'action');
} catch (error) {
  if (error instanceof AgentKernError) {
    console.error('Code:', error.code);
    console.error('Message:', error.message);
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| `AGENT_NOT_FOUND` | Agent ID does not exist |
| `POLICY_VIOLATION` | Action blocked by policy |
| `RESOURCE_LOCKED` | Resource is locked by another agent |
| `QUOTA_EXCEEDED` | Rate limit exceeded |
| `INVALID_REGION` | Invalid data region |

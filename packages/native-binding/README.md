# AgentKern Native Binding

Native Node.js bindings for AgentKern Rust core using NAPI-RS.

## Usage

```typescript
import { verifyAction, getAttestation, checkCarbonBudget } from '@agentkern/native';

// Verify an agent action
const result = await verifyAction({
  agentId: 'agent-123',
  action: 'transfer_funds',
  context: JSON.stringify({ amount: 1000, currency: 'USD' }),
});

if (result.allowed) {
  console.log('Action allowed');
} else {
  console.log('Blocked by:', result.blockingPolicies);
}

// Get TEE attestation
const attestation = await getAttestation('random-nonce-123');
console.log('Platform:', attestation.platform);
console.log('Quote:', attestation.quote);

// Check carbon budget
const allowed = checkCarbonBudget('agent-123', 50.0);
```

## Building

```bash
npm install
npm run build
```

## Architecture

This package uses NAPI-RS to expose Rust functions to Node.js:

- `verifyAction()` → `agentkern-gate::engine::GateEngine::verify()`
- `getAttestation()` → `agentkern-gate::tee::TeeRuntime::get_attestation()`
- `checkCarbonBudget()` → `agentkern-treasury::carbon::CarbonLedger`

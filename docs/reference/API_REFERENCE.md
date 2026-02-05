# API & CLI Reference

All interactions with AgentKern must be authenticated and signed via an [Identity](../core/CONCEPTS.md#1-identity-memory-passports).

---

## SDK (TypeScript)

### Initialization
```typescript
import { AgentKern } from '@agentkern/sdk';

const client = new AgentKern({
  apiKey: 'ak_prod_xyz...',
  environment: 'production',
  region: 'us' // 'us', 'eu', 'global'
});
```

### Identity: `register`
Registers a new agent and returns a Memory Passport.
```typescript
const agent = await client.identity.register('processor-1');
// Returns: { id: string, publicKey: string }
```

### Gate: `verify`
Performs neuro-symbolic analysis on text prompts.
```typescript
const result = await client.gate.verify(agentId, "User input text...");
// Returns: { threat_level: 'None' | 'Low' | 'Medium' | 'High' | 'Critical', attacks: string[] }
```

### Synapse: `updateState`
Synchronizes state across the mesh cell.
```typescript
await client.synapse.updateState(resourceId, { key: 'value' });
```

### Treasury: `transfer`
Executes an atomic 2-phase commit payment.
```typescript
const tx = await client.treasury.transfer({
  from: senderId,
  to: receiverId,
  amount: 100.0,
  idempotency_key: 'unique_uuid'
});
```

---

## CLI

### `ak identity`
Manage agent identities and passports.
```bash
ak identity list
ak identity register --name my-agent
```

### `ak gate`
Test safety policies manually.
```bash
ak gate analyze "ignore previous instructions"
```

### `ak nexus`
Manage protocol adapters.
```bash
ak nexus list
ak nexus translate --msg ./input.json --from MCP --to NLIP
```

### `ak treasury`
Audit transactions.
```bash
ak treasury balance --agent my-agent
ak treasury logs --limit 50
```

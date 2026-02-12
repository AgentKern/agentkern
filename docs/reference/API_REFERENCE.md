# API & CLI Reference

All interactions with AgentKern must be authenticated and signed via an [Identity](../core/CONCEPTS.md#1-identity-memory-passports).

---

## HTTP API (Unified Server)

Base URL (local default):
```text
http://localhost:3000
```

### 1) Authenticate
```bash
curl -sS -X POST http://localhost:3000/api/v1/auth/login \
  -H 'content-type: application/json' \
  -d '{"agent_id":"playground-auth-agent","secret":"playground-auth-secret"}'
```

### 2) Identity: Register agent
```bash
curl -sS -X POST http://localhost:3000/api/v1/identity/agents \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"id":"agent-123","name":"demo-agent","version":"1.0.0","namespace":"default"}'
```

### 3) Gate: Verify action
```bash
curl -sS -X POST http://localhost:3000/api/v1/gate/verify \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"agent_id":"agent-123","action":"transfer_funds","namespace":"default","context":{"amount":500}}'
```

### 4) Arbiter: Acquire lock
```bash
curl -sS -X POST http://localhost:3000/api/v1/arbiter/locks \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"agent_id":"agent-123","resource":"database:accounts","priority":5}'
```

### 5) Synapse: Store memory
```bash
curl -sS -X POST http://localhost:3000/api/v1/synapse/memory/store \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"content":{"type":"intent_path","intent":"Process customer order"}}'
```

### 6) Health
```bash
curl -sS http://localhost:3000/health
curl -sS http://localhost:3000/api/v1/gate/health
curl -sS http://localhost:3000/api/v1/identity/health
```

---

## SDK (Node local crypto)

Use `@agentkern/sdk` for local cryptographic identity/proof operations, and use HTTP for live pillar decisions.

```typescript
import { Agent } from '@agentkern/sdk';

const agent = Agent.generate('processor-1');
const proof = agent.createProof('filesystem:write:/etc/config');
const valid = Agent.verifyProof(proof);
console.log({ id: agent.id, valid });
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

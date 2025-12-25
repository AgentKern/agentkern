# Integration Guide

## Overview

This guide explains how to integrate AgentProof into your existing systems:
- **Agent Developers**: Add liability proofs to your AI agents
- **API Providers**: Verify proofs in your services
- **Platform Operators**: Monitor and manage trust

---

## For Agent Developers

### Step 1: Install SDK

```bash
npm install @agentproof/sdk
# or
pip install agentproof
```

### Step 2: Initialize Client

```typescript
import { AgentProofSDK } from '@agentproof/sdk';

const agentproof = new AgentProofSDK({
  apiUrl: process.env.AGENTPROOF_URL || 'https://api.agentproof.dev',
  agentId: 'my-agent-id',
  agentName: 'My AI Assistant',
  agentVersion: '1.0.0',
});
```

### Step 3: Request User Authorization

Before performing sensitive actions, request authorization:

```typescript
// User signs with their Passkey
const authorization = await agentproof.requestAuthorization({
  action: 'transfer',
  description: 'Transfer $500 to John',
  constraints: {
    maxAmount: 500,
    expiresIn: '5m',
  },
});
```

### Step 4: Create Proof for API Call

```typescript
const proof = await agentproof.createProof({
  authorization,
  intent: {
    action: 'transfer',
    target: {
      service: 'api.bank.com',
      endpoint: '/v1/transfers',
      method: 'POST',
    },
    parameters: { amount: 500, recipient: 'john@example.com' },
  },
});

// Include proof in request
await fetch('https://api.bank.com/v1/transfers', {
  headers: { 'X-AgentProof': proof.toHeader() },
  body: JSON.stringify({ amount: 500, recipient: 'john@example.com' }),
});
```

---

## For API Providers

### Step 1: Add Verification Middleware

```typescript
import { AgentProofVerifier } from '@agentproof/sdk';

const verifier = new AgentProofVerifier({
  apiUrl: 'https://api.agentproof.dev',
});

// Express middleware
app.use('/v1/transfers', async (req, res, next) => {
  const proofHeader = req.headers['x-agentproof'];
  
  if (!proofHeader) {
    return res.status(401).json({ error: 'Missing X-AgentProof header' });
  }

  const result = await verifier.verify(proofHeader);
  
  if (!result.valid) {
    return res.status(401).json({ error: result.error });
  }

  // Attach verification result to request
  req.agentproof = result;
  next();
});
```

### Step 2: Validate Intent Matches Request

```typescript
app.post('/v1/transfers', async (req, res) => {
  const { agentproof } = req;
  
  // Verify the proof authorizes THIS action
  if (agentproof.intent.action !== 'transfer') {
    return res.status(403).json({ error: 'Proof not valid for transfers' });
  }

  // Check amount is within constraints
  if (req.body.amount > agentproof.constraints.maxAmount) {
    return res.status(403).json({ error: 'Amount exceeds authorized limit' });
  }

  // Process the transfer
  await processTransfer(req.body);
  
  // Log for audit
  await logAudit({
    action: 'transfer',
    principal: agentproof.principal.id,
    agent: agentproof.agent.id,
    amount: req.body.amount,
    proofId: agentproof.proofId,
  });

  res.json({ success: true });
});
```

---

## For Platform Operators

### Dashboard Access

Access the management dashboard at:
```
http://localhost:3000/api/v1/dashboard/stats
```

### Key Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/dashboard/stats` | Overall statistics |
| `GET /api/v1/dashboard/trends` | Verification trends |
| `GET /api/v1/dashboard/top-agents` | Most active agents |
| `GET /api/v1/dashboard/policies` | Active policies |
| `POST /api/v1/dashboard/compliance/report` | Generate report |

### Policy Management

Create policies to control agent behavior:

```bash
curl -X POST http://localhost:3000/api/v1/dashboard/policies \
  -H "Content-Type: application/json" \
  -d '{
    "name": "High Value Limit",
    "description": "Require confirmation for amounts over $10,000",
    "rules": [{
      "name": "Confirm High Value",
      "condition": "amount > 10000",
      "action": "REQUIRE_CONFIRMATION"
    }]
  }'
```

---

## Trust Mesh Integration

For decentralized trust propagation:

### Join the Mesh

```typescript
import { TrustMeshNode } from '@agentproof/sdk';

const node = new TrustMeshNode({
  nodeId: 'my-node-id',
  bootstrapPeers: [
    'wss://mesh.agentproof.dev/node-1',
    'wss://mesh.agentproof.dev/node-2',
  ],
});

await node.connect();

// Listen for trust updates
node.on('trust-update', (update) => {
  console.log('Trust update received:', update);
});

// Broadcast trust changes
node.broadcast({
  type: 'trust-revoked',
  agentId: 'malicious-agent-id',
  reason: 'Violated constraints',
});
```

---

## Testing Integration

### Local Testing

```bash
# Start local AgentProof server
npm run start:dev

# Test proof creation
curl -X POST http://localhost:3000/api/v1/proof/create \
  -H "Content-Type: application/json" \
  -d '{
    "principal": {"id": "user-123"},
    "agent": {"id": "agent-456", "name": "Test Agent"},
    "intent": {"action": "test", "target": {"service": "test", "endpoint": "/test", "method": "GET"}}
  }'
```

### Integration Tests

```typescript
describe('AgentProof Integration', () => {
  it('should create and verify proof', async () => {
    const proof = await sdk.createProof(testOptions);
    const result = await sdk.verifyProof(proof.toHeader());
    expect(result.valid).toBe(true);
  });
});
```

---

## Security Checklist

- [ ] Use HTTPS in production
- [ ] Store secrets in environment variables
- [ ] Enable rate limiting
- [ ] Implement audit logging
- [ ] Set appropriate proof expiry times
- [ ] Validate all constraint parameters
- [ ] Handle proof verification failures gracefully

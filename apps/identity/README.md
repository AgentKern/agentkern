# AgentKern Identity

**Liability Infrastructure for the Agentic Economy**

> Not just authentication – cryptographic proof of WHO IS LIABLE when AI agents act.

---

## The Problem

AI agents are making decisions and taking actions on behalf of humans:
- Transferring money
- Accessing sensitive data
- Managing cloud infrastructure
- Making purchases

**But when something goes wrong, who is liable?**

Current solutions (Visa TAP, Mastercard Agent Pay) authenticate agents but don't solve liability.

---

## The Solution

AgentKern Identity provides **Liability Proofs** – cryptographic attestations that prove:

1. ✅ A specific human authorized a specific action
2. ✅ Authorization was made via hardware-bound Passkey (unforgeable)
3. ✅ The authorizer explicitly accepts liability
4. ✅ Clear constraints define what's authorized

---

## How It Works

```
┌──────────────┐     ┌─────────────────┐     ┌──────────────────┐
│    Human     │     │    AI Agent     │     │  Target Service  │
│  (Passkey)   │     │                 │     │   (Bank, API)    │
└──────┬───────┘     └────────┬────────┘     └────────┬─────────┘
       │                      │                       │
       │  1. Authorize        │                       │
       │  (Sign with Passkey) │                       │
       │─────────────────────▶│                       │
       │                      │                       │
       │                      │  2. Request + Proof   │
       │                      │──────────────────────▶│
       │                      │                       │
       │                      │     3. Verify locally │
       │                      │     (no API call)     │
       │                      │                       │
       │                      │◀──────────────────────│
       │                      │     4. Respond        │
```

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Passkey-Bound** | Only device owner can authorize |
| **Self-Verifying** | Target APIs verify locally – no latency |
| **Liability Shift** | Cryptographic proof of who accepts responsibility |
| **Universal** | Works for payments, data access, cloud ops, anything |
| **Decentralized** | Trust Mesh – no single point of failure |

---

## Quick Start

### Installation
```bash
npm install @agentkern/identity-sdk
```

### Create a Liability Proof
```typescript
import { AgentKernIdentity } from '@agentkern/identity-sdk';

const proof = await AgentKernIdentity.createProof({
  intent: {
    action: 'transfer',
    target: { service: 'api.bank.com', endpoint: '/v1/transfers' },
    parameters: { amount: 1000, currency: 'USD' }
  },
  constraints: {
    maxAmount: 5000,
    expiresIn: '5m'
  }
});

// Agent includes proof in request
fetch('https://api.bank.com/v1/transfers', {
  headers: {
    'X-AgentProof': proof.toHeader()
  }
});
```

### Verify a Proof (Target Service)
```typescript
import { AgentKernIdentity } from '@agentkern/identity-sdk';

const result = await AgentKernIdentity.verify(req.headers['x-agentkern-proof']);

if (result.valid) {
  // Proceed – liability is on the authorizer
  console.log(`Authorized by: ${result.principal.id}`);
} else {
  // Reject – no valid liability proof
  return res.status(401).json({ error: result.error });
}
```

---

## Architecture

### Four Pillars

1. **Proof-as-Header** – Self-verifying tokens in HTTP headers
2. **Trust Mesh** – Decentralized P2P trust network
3. **Intent DNS** – Global, cacheable trust resolution
4. **Embedded SDKs** – Zero-integration agent runtimes

---

## Quick Deployment

### Docker (Recommended)
```bash
# Clone repository
git clone https://github.com/AgentKern/agentkern.git
cd agentkern/apps/identity

# Start with Docker Compose
docker compose -f docker-compose.staging.yml up -d --build

# Verify health
curl http://localhost:3000/health
```

### Development
```bash
npm install
npm run start:dev

# Run tests
npm run test
npm run test:e2e
```

### Test Coverage
```bash
npm run test -- --coverage
# Current: 95% coverage, 319+ tests
```

---

## Documentation

- [Protocol Specification](docs/PROTOCOL_SPEC.md)
- [SDK Reference](docs/SDK.md)
- [Integration Guide](docs/INTEGRATION.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Trust Mesh Protocol](docs/TRUST_MESH_SPEC.md)

---

## Comparison

| Aspect | Visa TAP | AgentKern Identity |
|--------|----------|------------|
| **Focus** | Authentication | Liability |
| **Scope** | Payments only | Universal |
| **Architecture** | Centralized | Decentralized |
| **Integration** | Merchant work | Embedded in agents |
| **Proof Type** | Session-based | Self-verifying |

---

## License

MIT

---

**AgentKern Identity** – *Prove it. Own it. Trust it.*


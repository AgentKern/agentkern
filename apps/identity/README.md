# AgentKern Identity App

> **The unified TypeScript entry point for AgentKern.**

## Overview

`apps/identity` is the consolidated TypeScript application that provides:

- **Identity Pillar** - WebAuthn, proofs, trust management
- **Nexus Pillar** - Protocol translation (A2A, MCP, AgentKern)
- **Enterprise Features** - License gating, SSO integration

## Architecture

```
apps/identity/
├── controllers/
│   ├── proof.controller.ts      # Verifiable proofs
│   ├── dashboard.controller.ts  # Stats & metrics (enterprise)
│   ├── nexus.controller.ts      # Protocol translation
│   └── webauthn.controller.ts   # Passwordless auth
├── services/
│   ├── trust.service.ts         # Trust scoring
│   ├── nexus.service.ts         # Agent registry
│   └── webauthn.service.ts      # Credential management
├── guards/
│   └── enterprise-license.guard.ts  # Enterprise feature gating
└── entities/
    ├── trust-score.entity.ts    # TypeORM persistence
    └── webauthn-*.entity.ts     # Credential storage
```

## Rust Integration

Heavy-lifting operations call Rust via N-API:

```typescript
// Gate guardrails (0ms latency)
const bridge = require('../../../packages/bridge/index.node');
const result = bridge.attest(nonce);
```

See [ADR-001](../docs/adr/ADR-001-gateway-identity-merge.md) for architecture decision.

## Quick Start

```bash
# Install dependencies
pnpm install

# Start development server
cd apps/identity && npm run start:dev

# Run tests
npm test
```

## Endpoints

| Endpoint | Description |
|----------|-------------|
| `POST /api/v1/proofs/verify` | Verify a signed proof |
| `GET /api/v1/dashboard/stats` | System stats (enterprise) |
| `POST /nexus/agents` | Register an agent |
| `GET /nexus/protocols` | List supported protocols |
| `GET /.well-known/agent.json` | A2A agent card |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection | Required |
| `AGENTKERN_LICENSE_KEY` | Enterprise license | - |
| `EE_LICENSE_SERVICE_URL` | Enterprise license service | - |
| `IDENTITY_URL` | Public URL for this service | `http://localhost:3001` |

## Related

- [ARCHITECTURE.md](../docs/ARCHITECTURE.md) - System architecture
- [ADR-001](../docs/adr/ADR-001-gateway-identity-merge.md) - Gateway merge decision
- [packages/bridge](../packages/bridge) - N-API Rust bindings

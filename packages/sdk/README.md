# @agentkern/sdk

> **TypeScript SDK for AgentKern AI Agent Integration**

## Installation

```bash
npm install @agentkern/sdk
# or
pnpm add @agentkern/sdk
```

## Quick Start

### LangChain Integration

```typescript
import { AgentKernIdentityCallbackHandler } from '@agentkern/sdk';

// Create handler
const handler = new AgentKernIdentityCallbackHandler({
  principalId: 'my-agent-id',
  proofEndpoint: 'https://identity.agentkern.io/api/v1/proofs/verify',
});

// Use with LangChain
const llm = new ChatOpenAI({ callbacks: [handler] });
```

### Direct API

```typescript
import { AgentKernIdentityClient } from '@agentkern/sdk';

const client = new AgentKernIdentityClient({
  baseUrl: 'https://identity.agentkern.io',
  apiKey: process.env.AGENTKERN_API_KEY,
});

// Verify a proof
const result = await client.verifyProof({
  agentId: 'agent-123',
  action: 'transfer-funds',
  proof: '...',
});
```

## Features

- **LangChain Callbacks** - Automatic proof creation for tool calls
- **Trust Verification** - Verify agent trust scores
- **Protocol Translation** - A2A, MCP, AgentKern
- **WebAuthn** - Passwordless authentication

## API Reference

### `AgentKernIdentityCallbackHandler`

LangChain callback handler for AgentKern integration.

| Option | Type | Description |
|--------|------|-------------|
| `principalId` | `string` | Your agent's principal ID |
| `proofEndpoint` | `string` | AgentKern proof API endpoint |
| `autoProof` | `boolean` | Auto-create proofs for tools (default: true) |

### `AgentKernIdentityClient`

Direct client for AgentKern Identity API.

| Method | Description |
|--------|-------------|
| `verifyProof()` | Verify a signed proof |
| `getTrustScore()` | Get agent trust score |
| `registerAgent()` | Register with Nexus |

## License

MIT

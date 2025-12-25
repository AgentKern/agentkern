# SDK Reference

## Installation

### TypeScript/JavaScript
```bash
npm install @agentproof/sdk
```

### Python
```bash
pip install agentproof
```

---

## TypeScript SDK

### Creating a Liability Proof

```typescript
import { AgentProofSDK } from '@agentproof/sdk';

const sdk = new AgentProofSDK({
  apiUrl: 'https://api.agentproof.dev',
  agentId: 'your-agent-id',
  agentName: 'My AI Agent',
});

// Create a proof for an action
const proof = await sdk.createProof({
  principal: {
    id: 'user-123',
    publicKey: userPasskeyPublicKey,
  },
  intent: {
    action: 'transfer',
    target: {
      service: 'api.bank.com',
      endpoint: '/v1/transfers',
      method: 'POST',
    },
    parameters: {
      amount: 1000,
      currency: 'USD',
    },
  },
  constraints: {
    maxAmount: 5000,
    expiresIn: '5m',
    geoFence: ['US', 'CA'],
  },
  liability: {
    acceptedBy: 'principal',
    termsVersion: '1.0',
    disputeWindowHours: 72,
  },
});

// Use the proof in API requests
const response = await fetch('https://api.bank.com/v1/transfers', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'X-AgentProof': proof.toHeader(),
  },
  body: JSON.stringify({ amount: 1000, to: 'account-456' }),
});
```

### Verifying a Proof

```typescript
import { AgentProofSDK } from '@agentproof/sdk';

const sdk = new AgentProofSDK({ apiUrl: 'https://api.agentproof.dev' });

// Extract proof from incoming request
const proofHeader = req.headers['x-agentproof'];

const result = await sdk.verifyProof(proofHeader);

if (result.valid) {
  console.log('Proof verified!');
  console.log('Principal:', result.principal.id);
  console.log('Agent:', result.agent.name);
  console.log('Action:', result.intent.action);
} else {
  console.error('Invalid proof:', result.error);
}
```

---

## LangChain Integration

```typescript
import { AgentProofTool } from '@agentproof/sdk/langchain';
import { ChatOpenAI } from '@langchain/openai';
import { AgentExecutor, createOpenAIFunctionsAgent } from 'langchain/agents';

// Create AgentProof tool
const agentProofTool = new AgentProofTool({
  apiUrl: 'https://api.agentproof.dev',
  agentId: 'langchain-agent',
});

// Add to agent
const agent = await createOpenAIFunctionsAgent({
  llm: new ChatOpenAI({ model: 'gpt-4' }),
  tools: [agentProofTool],
  prompt: yourPrompt,
});

// Execute with automatic proof generation
const executor = new AgentExecutor({ agent, tools: [agentProofTool] });
await executor.invoke({ input: 'Transfer $500 to account 1234' });
```

---

## Python SDK

```python
from agentproof import AgentProofClient

client = AgentProofClient(
    api_url="https://api.agentproof.dev",
    agent_id="python-agent",
)

# Create proof
proof = client.create_proof(
    principal_id="user-123",
    action="transfer",
    target={
        "service": "api.bank.com",
        "endpoint": "/v1/transfers",
        "method": "POST"
    },
    parameters={"amount": 1000},
    constraints={"max_amount": 5000}
)

# Use in requests
import requests
response = requests.post(
    "https://api.bank.com/v1/transfers",
    headers={"X-AgentProof": proof.to_header()},
    json={"amount": 1000}
)
```

---

## API Reference

### `AgentProofSDK`

| Method | Description |
|--------|-------------|
| `createProof(options)` | Create a new liability proof |
| `verifyProof(header)` | Verify an existing proof |
| `revokeProof(proofId)` | Revoke a proof before expiry |

### Proof Options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `principal` | `Principal` | Yes | User creating the proof |
| `intent` | `Intent` | Yes | Action being authorized |
| `constraints` | `Constraints` | No | Limits on the authorization |
| `liability` | `Liability` | Yes | Liability terms |

### Verification Result

| Field | Type | Description |
|-------|------|-------------|
| `valid` | `boolean` | Whether proof is valid |
| `principal` | `Principal` | Who authorized the action |
| `agent` | `Agent` | Agent that created the proof |
| `intent` | `Intent` | What was authorized |
| `error` | `string?` | Error message if invalid |

---

## Error Handling

```typescript
try {
  const proof = await sdk.createProof(options);
} catch (error) {
  if (error.code === 'EXPIRED') {
    console.log('Proof expired, regenerate');
  } else if (error.code === 'UNAUTHORIZED') {
    console.log('User not authorized');
  } else {
    console.error('Unknown error:', error);
  }
}
```

---

## Best Practices

1. **Short Expiry** - Use 5-minute expiry for sensitive actions
2. **Specific Constraints** - Always set `maxAmount` for financial actions
3. **Audit Logging** - Log all proof verifications
4. **Error Recovery** - Handle proof expiry gracefully
5. **Secure Storage** - Never log full proof contents

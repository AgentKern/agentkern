# AgentKern SDK Usage Guide

AgentKern supports two complementary integration paths:
- **Local SDK crypto path** (Node/Python/Rust SDKs) for identity/proof operations.
- **Live HTTP pillar path** (`apps/server`) for Gate/Arbiter/Synapse/Identity runtime decisions.

Recommended production pattern: **HTTP-first with safe fallback behavior** if the server is unavailable.

## 1. Node.js SDK (local crypto + live HTTP)

### Installation
```bash
npm install @agentkern/sdk
```

### Local SDK Usage (Identity + Proofs)
```typescript
import { Agent } from '@agentkern/sdk';

// Generate a new agent with Ed25519 keys
const agent = Agent.generate('my-node-agent');

console.log(`Agent Created: ${agent.id}`);

// Create a Liability Proof for an action
const proof = agent.createProof('filesystem:write:/etc/config');

// Verify a proof
const isValid = Agent.verifyProof(proof);
console.log(`Proof is valid: ${isValid}`);
```

### Live HTTP Verification (Gate)
```typescript
const API_URL = process.env.AGENTKERN_API_URL ?? 'http://localhost:3000';

async function getAuthHeader() {
  const response = await fetch(`${API_URL}/api/v1/auth/login`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      agent_id: process.env.AGENTKERN_AUTH_AGENT_ID ?? 'playground-auth-agent',
      secret: process.env.AGENTKERN_AUTH_SECRET ?? 'playground-auth-secret',
    }),
  });
  const data = await response.json();
  return `${data.token_type ?? 'Bearer'} ${data.token}`;
}

async function verifyAction(agentId: string, action: string, context: Record<string, unknown>) {
  const auth = await getAuthHeader();
  const response = await fetch(`${API_URL}/api/v1/gate/verify`, {
    method: 'POST',
    headers: {
      authorization: auth,
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      agent_id: agentId,
      action,
      namespace: 'default',
      context,
    }),
  });
  return response.json();
}
```

---

## 2. Python SDK (local crypto + live HTTP)

### Installation
```bash
pip install agentkern
```

### Local SDK Usage
```python
from agentkern import Agent

# Create an agent
agent = Agent.generate("my-python-agent")
```

### Live HTTP Verification
```python
import os
import requests

base_url = os.getenv("AGENTKERN_API_URL", "http://localhost:3000")

auth_resp = requests.post(
    f"{base_url}/api/v1/auth/login",
    json={
        "agent_id": os.getenv("AGENTKERN_AUTH_AGENT_ID", "playground-auth-agent"),
        "secret": os.getenv("AGENTKERN_AUTH_SECRET", "playground-auth-secret"),
    },
).json()

headers = {
    "Authorization": f"{auth_resp.get('token_type', 'Bearer')} {auth_resp['token']}",
    "Content-Type": "application/json",
}

verify_resp = requests.post(
    f"{base_url}/api/v1/gate/verify",
    headers=headers,
    json={
        "agent_id": "agent-123",
        "action": "transfer_funds",
        "namespace": "default",
        "context": {"amount": 500},
    },
).json()

print(verify_resp)
```

---

## 3. Rust SDK (Core)

The native, zero-latency implementation.

### Installation
Add to `Cargo.toml`:
```toml
[dependencies]
agentkern-sdk-core = "0.1"
```

### Basic Usage
```rust
use agentkern_sdk_core::{Agent, LiabilityProof};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate agent
    let agent = Agent::generate("rust-agent")?;

    // Create proof
    let proof = agent.create_proof("consensus:vote")?;

    // Verify
    assert!(Agent::verify_proof(&proof)?);
    
    Ok(())
}
```

---

## 🛠️ Advanced: Security Guardrails (Coming Soon)

In future releases, all SDKs will support specialized **Prompt Guard** and **Context Guard** for AI-native safety directly in the client.

> [!NOTE]
> Today, production safety verification is performed via live HTTP APIs on the `Gate` pillar in the AgentKern Unified Server.

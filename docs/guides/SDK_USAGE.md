# AgentKern SDK Usage Guide

AgentKern provides multiple language bindings (SDKs) to interact with the Six Pillars. All SDKs are derived from the core Rust logic to ensure cryptographic consistency.

## 1. Node.js SDK (N-API)

Used in high-performance TypeScript/JavaScript applications.

### Installation
```bash
npm install @agentkern/sdk
```

### Basic Usage
```typescript
import { Agent, LiabilityProof } from '@agentkern/sdk';

// Generate a new agent with Ed25519 keys
const agent = Agent.generate('my-node-agent');

console.log(`Agent Created: ${agent.id}`);

// Create a Liability Proof for an action
const proof = agent.createProof('filesystem:write:/etc/config');

// Verify a proof
const isValid = Agent.verifyProof(proof);
console.log(`Proof is valid: ${isValid}`);
```

---

## 2. Python SDK (Maturin/PyO3)

Optimized for data science and AI agent integration.

### Installation
```bash
pip install agentkern
```

### Basic Usage
```python
from agentkern import Agent, GateEngine

# Create an agent
agent = Agent.generate("my-python-agent")

# Use the Gate Engine for verification
engine = GateEngine()

# Verify an action
allowed = engine.verify(
    agent_id=agent.id,
    action="api:call:openai",
    context={"tokens": 500}
)

if allowed:
    print("Action permitted by Gate!")
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
> Currently, safety verification should be performed via the `Gate` pillar in the AgentKern Unified Server.

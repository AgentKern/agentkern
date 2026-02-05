# Quick Start (10 minutes)

AgentKern provides cryptographic safety for autonomous AI agents. This guide covers your first verified interaction using the unified SDK.

> [!NOTE]
> Currently, the SDK exposes **Identity** and **Messaging** primitives. High-level safety checks are performed by the Rust server, which you communicate with via A2A messages.

## 1. Requirements
- **Node.js 18+**
- **Rust 1.75+** (if building from source)
- **Linux** (recommended for production TEE support)

## 2. Installation
```bash
npm install @agentkern/sdk
```

## 3. Minimal Working Example

This script generates a cryptographic identity, signs an action, and verifies the proof. This is the foundation of all AgentKern safety.

**File:** `index.ts`
```typescript
import { Agent } from '@agentkern/sdk';

async function main() {
  // 1. Generate a new Agent Identity (Ed25519)
  // In production, load this from a secure seed.
  const agent = Agent.generate('security-bot');
  console.log(`🆔 Agent Created: ${agent.id}`);
  console.log(`🔑 Public Key:    ${agent.publicKey}`);

  // 2. Create a Liability Proof for an action
  // This signs the intent "verify_prompt" with the hardware-backed key.
  const action = 'verify_prompt';
  const proof = agent.createProof(action);
  
  console.log(`\n📝 Signed Action: '${proof.action}'`);
  console.log(`📜 JWT Proof:     ${proof.jwt.substring(0, 30)}...`);

  // 3. Verify the Proof (Receiver Side)
  // This is what the Gate server does before executing any logic.
  const isValid = Agent.verifyProof(proof);

  if (isValid) {
    console.log(`\n✅ VERIFIED: Action is authentic and authorized.`);
  } else {
    console.error(`\n❌ FAILED: Invalid signature or expired proof.`);
  }
}

main().catch(console.error);
```

## 4. Run It
```bash
npx ts-node index.ts
```

## Expected Output
```text
🆔 Agent Created: security-bot_did:key:z6MkhaXgBZDvotDkL5257...
🔑 Public Key:    MCowBQYDK2VwAyEA...

📝 Signed Action: 'verify_prompt'
📜 JWT Proof:     eyJhbGciOiJFZERTQSIs...

✅ VERIFIED: Action is authentic and authorized.
```

---

## Next Steps

- **[Core Concepts](./core/CONCEPTS.md)** — Understand the Six Pillars.
- **[Common Workflows](./core/WORKFLOWS.md)** — Agent-to-Agent Messaging patterns.
- **[Contributing](./governance/CONTRIBUTING.md)** — Engineering standards.

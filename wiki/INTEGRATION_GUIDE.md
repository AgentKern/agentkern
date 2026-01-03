# AgentKern Integration Guide

> **Purpose**: How to connect your AI agents to AgentKern for governance, trust, and safety.

---

## Key Concept: AgentKern is Infrastructure, Not a Platform

**AgentKern does NOT build agents.** It provides governance for agents you build elsewhere.

Think of it like:
- **Stripe** → You build the store, Stripe handles payments
- **Auth0** → You build the app, Auth0 handles authentication
- **AgentKern** → You build the agent, AgentKern handles governance

```
┌─────────────────────────────────────────────────────────────────┐
│                    YOUR EXISTING AGENTS                         │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐       │
│  │  AutoGPT      │  │  LangChain    │  │  CrewAI       │       │
│  │  Agent        │  │  Agent        │  │  Agent        │       │
│  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘       │
│          └──────────────────┼──────────────────┘                │
│                             │ SDK / HTTP API                    │
└─────────────────────────────│───────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        AgentKern                                │
│   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│   │ Identity │ │   Gate   │ │ Treasury │ │  Nexus   │          │
│   └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Table of Contents

1. [Integration Patterns](#1-integration-patterns)
2. [Quick Start](#2-quick-start)
3. [SDK Reference](#3-sdk-reference)
4. [HTTP API Reference](#4-http-api-reference)
5. [Agent Registration](#5-agent-registration)
6. [The Action Lifecycle](#6-the-action-lifecycle)
7. [Liability Proofs](#7-liability-proofs)
8. [Error Handling](#8-error-handling)
9. [Best Practices](#9-best-practices)

---

## 1. Integration Patterns

### Pattern A: SDK Integration (Recommended)

Embed AgentKern checks directly in your agent code.

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│  Your Agent │ ──SDK──▶│  AgentKern  │         │  External   │
│             │◀────────│             │         │  Service    │
│             │         └─────────────┘         └─────────────┘
│             │────────────────────────────────────▶│
└─────────────┘   (If allowed, proceed with action) │
```

**Best for**: Full control, custom logic, new projects.

### Pattern B: Proxy/Sidecar

All agent traffic passes through AgentKern automatically.

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│  Your Agent │ ───────▶│  AgentKern  │ ───────▶│  External   │
│             │◀────────│    Proxy    │◀────────│  Service    │
└─────────────┘         └─────────────┘         └─────────────┘
                        (Intercepts, checks,
                         logs everything)
```

**Best for**: Legacy agents, zero-code integration, centralized control.

### Pattern C: HTTP API Direct

Call AgentKern endpoints directly via REST.

**Best for**: Non-standard languages, testing, simple integrations.

---

## 2. Quick Start

### Step 1: Register Your Agent

```typescript
// First time only - register your agent with AgentKern
const response = await fetch('https://api.agentkern.io/agents', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    id: 'my-langchain-agent',
    name: 'My LangChain Agent',
    version: '1.0.0',
  }),
});

const agent = await response.json();
// Agent registered with starting trust score = 500
```

### Step 2: Check Before Acting

```typescript
// Before every significant action
const check = await fetch('https://api.agentkern.io/agents/check-action', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    agentId: 'my-langchain-agent',
    action: 'send_email',
    target: {
      service: 'sendgrid',
      endpoint: '/v3/mail/send',
      method: 'POST',
    },
    estimatedCost: 0.001,
  }),
});

const result = await check.json();

if (result.allowed) {
  // ✅ Proceed with the action
  await sendEmail(to, subject, body);
  
  // Report success
  await fetch('https://api.agentkern.io/agents/record-success', {
    method: 'POST',
    headers: { 'Authorization': 'Bearer YOUR_API_KEY' },
    body: JSON.stringify({ agentId: 'my-langchain-agent' }),
  });
} else {
  // ❌ Action blocked
  console.log('Blocked:', result.reason);
  // Maybe: "Reputation too low" or "Budget exceeded"
}
```

---

## 3. SDK Reference

### Available SDKs

| Language | Package | Status |
|----------|---------|--------|
| TypeScript/Node | `@agentkern/sdk` | 🟢 Available |
| Python | `agentkern` | 🟡 Planned |
| Go | `github.com/agentkern/go-sdk` | 🟡 Planned |
| Rust | `agentkern` | 🟡 Planned |

### TypeScript SDK Usage

```typescript
import { AgentKern } from '@agentkern/sdk';

// Initialize
const kern = new AgentKern({
  apiKey: process.env.AGENTKERN_API_KEY,
  agentId: 'my-agent',
});

// Check action
const result = await kern.checkAction({
  action: 'database_write',
  target: { service: 'postgres', endpoint: '/users', method: 'INSERT' },
});

// Record outcomes
await kern.recordSuccess();
await kern.recordFailure('Connection timeout');

// Get trust score
const trust = await kern.getTrustScore();
console.log(`Trust: ${trust.score}/100 (${trust.level})`);
```

---

## 4. HTTP API Reference

### Base URL

```
Production: https://api.agentkern.io
Local:      http://localhost:3000
```

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/agents` | POST | Register new agent |
| `/agents/:id` | GET | Get agent status |
| `/agents/check-action` | POST | Check if action is allowed |
| `/agents/record-success` | POST | Report successful action |
| `/agents/record-failure` | POST | Report failed action |
| `/agents/:id/suspend` | POST | Suspend an agent |
| `/agents/:id/terminate` | POST | Permanently terminate |
| `/trust/:agentId` | GET | Get trust score |
| `/proofs/verify` | POST | Verify liability proof |

### Authentication

```bash
# API Key in header
Authorization: Bearer ak_live_xxxxxxxxxxxxx

# Or Liability Proof for action verification
X-AgentKern-Proof: BASE64_ENCODED_PROOF
```

---

## 5. Agent Registration

### What Happens When You Register

1. Agent gets unique ID (or uses your provided ID)
2. Starting trust score = 500 (on 0-1000 scale)
3. Default budget assigned:
   - 1,000,000 tokens/day
   - 10,000 API calls/day
   - $100/day cost limit
4. Status set to ACTIVE

### Registration Request

```json
POST /agents
{
  "id": "my-agent-unique-id",     // Optional, auto-generated if omitted
  "name": "My Trading Agent",
  "version": "1.0.0",
  "customBudget": {               // Optional overrides
    "maxTokens": 500000,
    "maxCostUsd": 50
  }
}
```

### Registration Response

```json
{
  "id": "my-agent-unique-id",
  "name": "My Trading Agent",
  "version": "1.0.0",
  "status": "ACTIVE",
  "reputation": {
    "score": 500,
    "successfulActions": 0,
    "failedActions": 0,
    "violations": 0
  },
  "budget": {
    "maxTokens": 500000,
    "maxApiCalls": 10000,
    "maxCostUsd": 50,
    "periodSeconds": 86400
  },
  "createdAt": "2025-12-31T17:00:00Z"
}
```

---

## 6. The Action Lifecycle

Every agent action should follow this pattern:

```
┌──────────────────────────────────────────────────────────────┐
│                    ACTION LIFECYCLE                          │
└──────────────────────────────────────────────────────────────┘

    ┌─────────┐
    │  START  │
    └────┬────┘
         │
         ▼
┌─────────────────┐
│ 1. checkAction()│ ─────────────────┐
└────────┬────────┘                  │
         │                           │
    ┌────┴────┐                      │
    │ allowed?│                      │
    └────┬────┘                      │
    YES  │  NO                       │
         │   └──────────────────────▶ Log & abort
         ▼
┌─────────────────┐
│ 2. Perform the  │
│    action       │
└────────┬────────┘
         │
    ┌────┴────┐
    │ success?│
    └────┬────┘
    YES  │  NO
         │   │
         ▼   ▼
┌──────────┐ ┌──────────────┐
│ record   │ │ record       │
│ Success()│ │ Failure()    │
└──────────┘ └──────────────┘
```

### Code Example

```typescript
async function executeAgentAction(action: string, params: any) {
  // Step 1: Check permission
  const check = await kern.checkAction({
    action,
    target: params.target,
    estimatedCost: params.estimatedCost,
  });

  if (!check.allowed) {
    console.log(`Blocked: ${check.reason}`);
    return { success: false, reason: check.reason };
  }

  // Step 2: Perform action
  try {
    const result = await performAction(action, params);
    
    // Step 3a: Report success
    await kern.recordSuccess({
      tokensUsed: result.tokensUsed,
      cost: result.cost,
    });
    
    return { success: true, result };
  } catch (error) {
    // Step 3b: Report failure
    await kern.recordFailure(error.message);
    
    return { success: false, error: error.message };
  }
}
```

---

## 7. Liability Proofs

For high-value actions, AgentKern requires a **Liability Proof** — a cryptographically signed statement of who accepts responsibility.

### When Required

- Transactions over $100
- Database modifications
- External API calls to sensitive services
- Actions requiring human approval

### Proof Structure

```typescript
// Client-side: Create and sign proof
const proof = {
  version: 'v1',
  payload: {
    proofId: crypto.randomUUID(),
    principal: { id: 'user-123', credentialId: 'cred-456' },
    agent: { id: 'my-agent', name: 'Trading Bot' },
    intent: {
      action: 'transfer_funds',
      target: { service: 'bank-api', endpoint: '/transfer', method: 'POST' },
    },
    liability: { acceptedBy: 'user-123' },
    issuedAt: new Date().toISOString(),
    expiresAt: new Date(Date.now() + 3600000).toISOString(), // 1 hour
  },
  signature: 'ES256_SIGNATURE_HERE',
};

// Send with request
headers['X-AgentKern-Proof'] = btoa(JSON.stringify(proof));
```

---

## 8. Error Handling

### Common Error Responses

| Status | Reason | Action |
|--------|--------|--------|
| `allowed: false, reason: "Global kill switch activated"` | Emergency shutdown | Wait for system restore |
| `allowed: false, reason: "Agent is SUSPENDED"` | Too many violations | Contact admin |
| `allowed: false, reason: "Reputation too low"` | Score < 100 | Build trust via successful actions |
| `allowed: false, reason: "Budget exceeded"` | Daily limit hit | Wait for period reset |
| `allowed: false, reason: "Rate limit exceeded"` | Too many requests | Slow down, retry later |
| `allowed: false, reason: "Prompt injection detected"` | Malicious input | Clean input, try again |

### Retry Strategy

```typescript
async function checkWithRetry(request, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    const result = await kern.checkAction(request);
    
    if (result.allowed) return result;
    
    // Non-retryable errors
    if (['Global kill switch', 'SUSPENDED', 'TERMINATED'].some(
      r => result.reason.includes(r)
    )) {
      throw new Error(result.reason);
    }
    
    // Retryable: rate limit
    if (result.reason.includes('Rate limit')) {
      await sleep(1000 * (i + 1)); // Exponential backoff
      continue;
    }
    
    return result; // Other errors, don't retry
  }
}
```

---

## 9. Best Practices

### DO ✅

- **Check before every significant action** — Not just expensive ones
- **Report all outcomes** — Success AND failure, for accurate trust scoring
- **Use idempotency keys** — For retryable actions
- **Handle kill switch gracefully** — Save state, clean up
- **Log proof IDs** — For audit trail correlation

### DON'T ❌

- **Don't skip checks for "safe" actions** — Trust scoring needs data
- **Don't cache check results** — Conditions change rapidly
- **Don't ignore low trust warnings** — Fix issues before blocked
- **Don't share API keys** — Each agent should have its own
- **Don't retry violations** — They're logged, repeated attempts look worse

---

## Next Steps

1. [Get an API key](https://agentkern.io/signup) (when available)
2. Install the SDK for your language
3. Register your agent
4. Implement the action lifecycle pattern
5. Monitor trust score in dashboard

---

## Questions?

- **Architecture**: See [IDENTITY_DESIGN.md](IDENTITY_DESIGN.md)
- **Trust scoring**: See [Identity Pillar § Trust Scoring](IDENTITY_DESIGN.md#3-trust-scoring-system)
- **Policy engine**: See [Gate Pillar](GATE_DESIGN.md) (coming soon)

---

*Last updated: 2025-12-31*

# Identity Pillar - Design Wiki

> **Purpose**: Comprehensive documentation for understanding the Identity pillar.
> **Audience**: Developers learning the codebase, future maintainers, contributors.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Core Concepts](#2-core-concepts)
3. [Trust Scoring System](#3-trust-scoring-system)
4. [Agent Sandbox](#4-agent-sandbox)
5. [Agent Lifecycle](#5-agent-lifecycle)
6. [Design Decisions](#6-design-decisions)
7. [File Reference](#7-file-reference)
8. [Common Confusions](#8-common-confusions)
9. [API Reference](#9-api-reference)
10. [WebAuthn / Passkeys](#10-webauthn--passkeys)
11. [Liability Proofs](#11-liability-proofs)
12. [DNS-Style Trust Resolution](#12-dns-style-trust-resolution)
13. [Nexus Integration](#13-nexus-integration)
14. [Security & Audit Logging](#14-security--audit-logging)
15. [Database Entities](#15-database-entities)
16. [Module Organization](#16-module-organization)
17. [Complete File Map](#17-complete-file-map)

---

## 1. Overview

The Identity pillar is the "passport office" for AI agents. It answers three questions:

| Question | How We Answer It |
|----------|------------------|
| **Who is this agent?** | Verifiable Credentials (W3C standard) |
| **Can I trust them?** | Trust Score (0-100 scale, persisted) |
| **Can they do this action?** | Agent Sandbox (budgets, rate limits, kill switch) |

### Technology Stack

- **Framework**: NestJS (TypeScript)
- **Database**: PostgreSQL via TypeORM
- **Auth**: WebAuthn (hardware keys, biometrics)

---

## 2. Core Concepts

### 2.1 What is an "Agent"?

An **agent** is any autonomous AI program that performs actions through AgentKern:
- ChatGPT plugins
- AutoGPT instances
- Custom AI assistants
- Inter-agent services

### 2.2 What is a "Request"?

When an agent wants to do something (call an API, access a database, pay another agent), it creates a **request**:

```typescript
const request: SandboxActionRequest = {
  agentId: 'agent-123',          // Who is asking
  action: 'llm_call',            // What type of action
  target: {
    service: 'openai',           // Which service
    endpoint: '/v1/completions', // Which endpoint
    method: 'POST'               // HTTP method
  },
  estimatedTokens: 1000,         // Resource estimate
  estimatedCost: 0.02            // Cost estimate
};
```

AgentKern's `checkAction()` function decides: **ALLOW** or **BLOCK**.

---

## 3. Trust Scoring System

### 3.1 Two Scales (Important!)

There are **two different scales** in the codebase:

| Service | Scale | Starting Value | Purpose |
|---------|-------|----------------|---------|
| **TrustService** | 0-100 | 50 | Long-term reputation, persisted |
| **AgentSandboxService** | 0-1000 | 500 | Runtime decisions, in-memory |

**Conversion**: `sandboxScore = trustScore × 10`

### 3.2 Trust Levels

```
TrustService Scale (0-100):

  0 ──── 25 ──── 50 ──── 75 ──── 100
  │      │       │       │       │
UNTRUSTED │    MEDIUM   HIGH  VERIFIED
         LOW

AgentSandboxService Scale (0-1000):

  0 ─── 100 ─────── 500 ─────────── 1000
  │      │           │               │
BLOCKED  │       NEW AGENT       MAXIMUM
  (threshold)   (can operate)
```

### 3.3 How Trust Changes

| Event | TrustService (0-100) | AgentSandbox (0-1000) |
|-------|---------------------|----------------------|
| New agent registers | +50 (starting) | +500 (starting) |
| Successful action | +5 | +50 |
| Failed action | -10 | -100 |
| Policy violation | -25 | -250 |
| Peer endorsement | +5 | +50 |
| Credential verified | +10 | +100 |

### 3.4 Weighted Factors (TrustService)

The final trust score is calculated from multiple factors:

```typescript
WEIGHTS = {
  transactionSuccess: 0.35,  // 35% - Did they complete transactions?
  responseTime: 0.15,        // 15% - Are they fast?
  policyCompliance: 0.25,    // 25% - Do they follow rules?
  peerEndorsements: 0.10,    // 10% - Do others vouch for them?
  accountAge: 0.10,          // 10% - How long have they existed?
  credentials: 0.05,         // 5%  - Do they have verified credentials?
}
```

---

## 4. Agent Sandbox

The sandbox enforces **runtime safety** — preventing agents from causing harm.

### 4.1 Safety Mechanisms

| Mechanism | What It Does | Default Value |
|-----------|--------------|---------------|
| **Kill Switch** | Emergency stop for ALL agents | OFF |
| **Rate Limit** | Max requests per minute | 100/min |
| **Token Budget** | Max tokens per day | 1,000,000 |
| **API Call Budget** | Max API calls per day | 10,000 |
| **Cost Budget** | Max $ spend per day | $100 |
| **Reputation Threshold** | Min score to operate | 100 (on 0-1000 scale) |
| **Violation Threshold** | # violations before auto-suspend | 3 |

### 4.2 The `checkAction()` Flow

This is the **most important function** — every agent request goes through it:

```
Agent wants to do something
        │
        ▼
┌─────────────────────────────────┐
│ 1. Check Global Kill Switch    │ ─── ON? ──→ BLOCK (all agents)
└─────────────────────────────────┘
        │ OFF
        ▼
┌─────────────────────────────────┐
│ 2. Get or create agent record  │ (auto-register if unknown)
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│ 3. Check agent status          │ ─── Not ACTIVE ──→ BLOCK
└─────────────────────────────────┘
        │ ACTIVE
        ▼
┌─────────────────────────────────┐
│ 4. Check rate limit            │ ─── Exceeded ──→ RATE_LIMITED
└─────────────────────────────────┘
        │ OK
        ▼
┌─────────────────────────────────┐
│ 5. Reset budget if period ended│ (auto-resets daily)
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│ 6. Check budget                │ ─── Exceeded ──→ BUDGET_EXCEEDED
└─────────────────────────────────┘
        │ OK
        ▼
┌─────────────────────────────────┐
│ 7. Check reputation            │ ─── < 100 ──→ BLOCK
└─────────────────────────────────┘
        │ OK
        ▼
┌─────────────────────────────────┐
│ 8. Prompt injection guard      │ ─── High/Critical ──→ VIOLATION + BLOCK
└─────────────────────────────────┘
        │ OK
        ▼
     ✅ ALLOW
```

### 4.3 In-Flight Requests

**Definition**: A request that has started but hasn't finished yet.

**Kill Switch Behavior**:
- New requests → **Blocked immediately**
- In-flight requests → **Allowed to complete**

**Why?** Interrupting mid-transaction could corrupt external systems (e.g., half-completed payment).

---

## 5. Agent Lifecycle

```
                        AGENT LIFECYCLE
                        
┌────────────────────────────────────────────────────────────────┐
│                      1. REGISTER                               │
│   • Agent joins the system                                     │
│   • Gets status = ACTIVE                                       │
│   • Gets reputation = 500 (on 0-1000 scale)                    │
│   • Gets default budget (1M tokens, 10K calls, $100/day)       │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                       2. OPERATE                               │
│   • Makes requests via checkAction()                           │
│   • On success: recordSuccess() → +1 reputation                │
│   • On failure: recordFailure() → -10 reputation               │
│   • On violation: recordViolation() → -100 reputation          │
└────────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│   RATE_LIMITED   │ │  BUDGET_EXCEEDED │ │    SUSPENDED     │
│   (temporary)    │ │   (temporary)    │ │  (needs action)  │
│                  │ │                  │ │                  │
│ Auto-recovers    │ │ Auto-recovers    │ │ Needs manual     │
│ after success    │ │ on period reset  │ │ reactivation     │
└──────────────────┘ └──────────────────┘ └──────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                      4. TERMINATED                             │
│   • Kill switch OR 3+ violations                               │
│   • PERMANENT — cannot be reactivated                          │
│   • All future requests blocked                                │
└────────────────────────────────────────────────────────────────┘
```

### 5.1 Status Transitions

| From | To | Trigger | Recovery |
|------|----|---------|----------|
| ACTIVE | RATE_LIMITED | >100 req/min | Auto (on next success) |
| ACTIVE | BUDGET_EXCEEDED | Budget depleted | Auto (on period reset) |
| ACTIVE | SUSPENDED | 3 violations OR manual | Manual (`reactivateAgent()`) |
| ANY | TERMINATED | Kill switch OR manual | **None** (permanent) |

---

## 6. Design Decisions

### 6.1 Why Two Trust Scales?

| Decision | Rationale |
|----------|-----------|
| TrustService (0-100) | Standard, human-readable, maps to trust levels |
| AgentSandbox (0-1000) | More granular for runtime decisions, allows fine-tuning |

### 6.2 Why Asymmetric Reputation (Slow Gain, Fast Loss)?

```
+1 per success  vs  -10 per failure
```

**Rationale**: Building trust should be HARD. Losing it should be FAST.
- Prevents gaming: Can't offset bad behavior with volume
- Matches real-world: One scandal destroys years of reputation

### 6.3 Why 500 Starting Score (Not 0)?

**Rationale**: New agents need to operate immediately.
- Score of 0 = blocked (below 100 threshold)
- Score of 500 = can operate, but has room to fall
- One violation (500 - 100 = 400) still allows operation

### 6.4 Why 3 Violations = Auto-Suspend?

**Rationale**:
- 1 violation = could be accident
- 2 violations = concerning pattern
- 3 violations = clearly problematic → auto-suspend

### 6.5 Why In-Flight Requests Aren't Interrupted?

**Rationale**: Safety over speed.
- Mid-transaction interruption = potential data corruption
- Example: Agent halfway through bank transfer → interruption = money lost
- New requests are blocked immediately; in-flight complete then stop

---

## 7. File Reference

### 7.1 Directory Structure

```
apps/identity/
├── src/
│   ├── app.module.ts           # Main NestJS module
│   ├── main.ts                 # Entry point
│   ├── controllers/
│   │   ├── agents.controller.ts     # Agent CRUD endpoints
│   │   ├── dashboard.controller.ts  # Admin UI endpoints
│   │   ├── proof.controller.ts      # Credential verification
│   │   └── webauthn.controller.ts   # Hardware auth
│   ├── services/
│   │   ├── trust.service.ts         # ⭐ Trust scoring (606 lines)
│   │   ├── agent-sandbox.service.ts # ⭐ Runtime safety (867 lines)
│   │   ├── webauthn.service.ts      # Hardware auth
│   │   ├── gate.service.ts          # Rust bridge (prompt guard)
│   │   └── audit-logger.service.ts  # Security logging
│   ├── entities/                     # TypeORM database models
│   │   ├── agent-record.entity.ts
│   │   ├── trust-score.entity.ts
│   │   └── trust-event.entity.ts
│   └── domain/                       # Business logic interfaces
│       └── agent.entity.ts           # AgentRecord, AgentStatus types
├── test/                             # E2E and unit tests
└── docker-compose.yml                # Local dev database
```

### 7.2 Key Files and Their Purpose

| File | Purpose | Key Functions |
|------|---------|---------------|
| `trust.service.ts` | Long-term reputation management | `getTrustScore()`, `recordEvent()`, `recalculateTrustScore()` |
| `agent-sandbox.service.ts` | Runtime safety enforcement | `checkAction()`, `recordSuccess()`, `recordViolation()`, `activateGlobalKillSwitch()` |
| `gate.service.ts` | Bridge to Rust policy engine | `guardPrompt()`, `verify()` |
| `webauthn.service.ts` | Hardware authentication | `generateRegistration()`, `verifyAuthentication()` |
| `audit-logger.service.ts` | Security event logging | `logSecurityEvent()` |

---

## 8. Common Confusions

### 8.1 "Score 50 — Can They Operate?"

**Answer**: YES, but check which scale!

| If You Mean | Scale | Starting Value | Threshold | Can Operate? |
|-------------|-------|----------------|-----------|--------------|
| TrustService score = 50 | 0-100 | 50 | N/A (no threshold in TrustService) | N/A |
| AgentSandbox score = 50 | 0-1000 | 500 | 100 | NO (50 < 100) |
| AgentSandbox score = 500 | 0-1000 | 500 | 100 | YES (500 > 100) |

**New agents start with sandbox score = 500, so they CAN operate.**

### 8.2 "How Does a Low-Score Agent Build Trust?"

**Answer**: They start with 500, not 0.

If somehow an agent reaches score < 100:
- They're blocked until score recovers
- Admin can manually boost score or reactivate
- This is by design — severe punishment for bad behavior

### 8.3 "What's the Difference Between TrustService and AgentSandboxService?"

| Aspect | TrustService | AgentSandboxService |
|--------|--------------|---------------------|
| **Scope** | Long-term reputation | Runtime safety |
| **Scale** | 0-100 | 0-1000 |
| **Storage** | Database (persists) | In-memory (with DB backup) |
| **Focus** | History, factors, credentials | Budgets, limits, kills switch |
| **Checks** | N/A (just tracking) | Every request goes through |

## 10. WebAuthn / Passkeys

Hardware-based authentication for humans or systems controlling agents.

### 10.1 What is WebAuthn?

WebAuthn allows **passwordless login** using:
- **Hardware keys** (YubiKey, Titan Key)
- **Biometrics** (TouchID, FaceID, fingerprint)
- **Platform authenticators** (Windows Hello)

### 10.2 The Flow

```
┌─────────────┐    1. Request Registration    ┌─────────────┐
│   Browser/  │ ─────────────────────────────▶│   Identity  │
│   Device    │                               │   Service   │
└─────────────┘                               └─────────────┘
       │                                             │
       │ 2. Challenge returned                       │
       ◀─────────────────────────────────────────────┘
       │
       │ 3. User touches hardware key / uses biometric
       │
       ▼
┌─────────────┐    4. Signed response         ┌─────────────┐
│  Passkey    │ ─────────────────────────────▶│   Identity  │
│  Created    │                               │   Service   │
└─────────────┘                               └─────────────┘
                                                     │
                                              5. Store credential
                                                     │
                                                     ▼
                                              ┌─────────────┐
                                              │  Database   │
                                              └─────────────┘
```

### 10.3 Key Service Methods

| Method | Purpose |
|--------|---------|
| `generateRegistrationOptions()` | Create challenge for new device |
| `verifyRegistration()` | Validate and store credential |
| `generateAuthenticationOptions()` | Create challenge for login |
| `verifyAuthentication()` | Validate login attempt |
| `revokeCredential()` | Disable a credential |

### 10.4 File: `services/webauthn.service.ts` (422 lines)

---

## 11. Liability Proofs

Cryptographically signed statements proving who authorized an agent action.

### 11.1 What is a Liability Proof?

When an agent acts, **someone must accept liability**. A Liability Proof is a signed document saying:

> "I, Principal X, authorize Agent Y to perform Action Z under Constraints C"

### 11.2 Proof Structure

```typescript
interface LiabilityProof {
  version: 'v1';
  payload: {
    proofId: string;
    principal: { id: string; credentialId: string };
    agent: { id: string; name: string };
    intent: { 
      action: string; 
      target: { service: string; endpoint: string } 
    };
    constraints?: {
      maxCost?: number;
      validHours?: { start: number; end: number };
      geoFence?: string[];
    };
    liability: { acceptedBy: string };
    issuedAt: string;
    expiresAt: string;
  };
  signature: string;  // ES256 (ECDSA P-256)
}
```

### 11.3 Verification Flow

```
1. Parse proof from X-AgentKernIdentity header
2. Check expiration (expiresAt > now)
3. Check issuedAt not in future
4. Verify cryptographic signature (ES256)
5. Validate constraints (time, geo, cost)
6. Return result with liability info
```

### 11.4 Key Files

| File | Purpose |
|------|---------|
| `services/proof-signing.service.ts` | Create and sign proofs |
| `services/proof-verification.service.ts` | Verify proofs |
| `domain/liability-proof.entity.ts` | Type definitions |

---

## 12. DNS-Style Trust Resolution

Global, cacheable lookup for agent-principal trust relationships.

### 12.1 Concept

Like DNS resolves domain names to IP addresses, Trust DNS resolves:
```
(agentId, principalId) → TrustResolution
```

### 12.2 Trust Resolution Response

```typescript
interface TrustResolution {
  agentId: string;
  principalId: string;
  trusted: boolean;           // Can this agent act for this principal?
  trustScore: number;         // 0-1000
  ttl: number;                // Cache time in seconds
  flags: {
    revoked: boolean;
    suspended: boolean;
  };
}
```

### 12.3 Caching Strategy

- **Short TTL** (60s) for low-trust relationships
- **Long TTL** (3600s) for high-trust relationships
- **Immediate invalidation** on failure/revocation

### 12.4 File: `services/dns-resolution.service.ts` (271 lines)

---

## 13. Nexus Integration

Protocol translation layer for inter-agent communication.

### 13.1 Purpose

Agents from different vendors use different protocols:
- **A2A** (Google)
- **MCP** (Anthropic)
- **NLIP** (ECMA)

Nexus translates between them.

### 13.2 Key Capabilities

| Function | Description |
|----------|-------------|
| `registerAgent()` | Add agent to registry |
| `discoverAgent()` | Fetch agent card from URL |
| `routeTask()` | Find best agent for a task |
| `translateMessage()` | Convert between protocols |

### 13.3 Agent Discovery

Agents publish their capabilities at:
```
https://agent.example.com/.well-known/agent.json
```

Nexus fetches this to learn what an agent can do.

### 13.4 File: `services/nexus.service.ts` (262 lines)

---

## 14. Security & Audit Logging

Every security-relevant event is logged immutably.

### 14.1 Audit Event Types

| Type | When Logged |
|------|-------------|
| `KILL_SWITCH_ACTIVATED` | Global kill switch triggered |
| `AGENT_SUSPENDED` | Agent suspended for violations |
| `PROOF_VERIFICATION_FAILURE` | Invalid signature detected |
| `KEY_REVOKED` | Trust relationship revoked |
| `SUSPICIOUS_ACTIVITY` | Anomaly detected |
| `SECURITY_ALERT` | Critical security event |

### 14.2 What Gets Logged

```typescript
{
  id: uuid,
  type: 'SECURITY_ALERT',
  timestamp: '2025-12-31T14:00:00Z',
  agentId: 'agent-123',
  principalId: 'user-456',
  success: false,
  metadata: { reason: 'Prompt injection detected', score: 350 }
}
```

### 14.3 File: `services/audit-logger.service.ts`

### 14.4 Hardware Enclave (TEE) Verification

Identity leverages **Confidential Computing** (TEE) to protect sensitive operations.

- **Implementation**: `gate.service.ts` links to the Rust `gate` crate via N-API.
- **Function**: `attest(nonce)` generates a hardware-signed quote proving the code is running in a genuine enclave (e.g., Intel SGX/TDX).

```typescript
// Attestation Structure
interface Attestation {
  platform: string;   // 'sgx', 'tdx', 'sev'
  quote: number[];    // Hardware signature
  measurement: number[]; // Code hash
  timestamp: number;
}
```

---

## 15. Database Entities

All persistent data is stored via TypeORM.

### 15.1 Entity Reference

| Entity | Table | Purpose |
|--------|-------|---------|
| `AgentRecordEntity` | `agents` | Agent registration, budget, reputation (JSONB) |
| `TrustRecordEntity` | `trust_records` | TrustService scores & Agent-Principal links |
| `TrustEventEntity` | `trust_events` | History of trust-affecting events |
| `AuditEventEntity` | `audit_events` | Security audit trail |
| `WebAuthnCredentialEntity` | `webauthn_credentials` | Passkey/hardware key storage |
| `WebAuthnChallengeEntity` | `webauthn_challenges` | Pending auth challenges |
| `VerificationKeyEntity` | `verification_keys` | Public keys for proof verification |
| `SystemConfigEntity` | `system_config` | Global settings (kill switch state) |

### 15.2 Location: `entities/` directory

---

## 16. Module Organization

NestJS modules group related functionality.

| Module | Services | Purpose |
|--------|----------|---------|
| `DatabaseModule` | TypeORM | Database connection and entities |
| `SecurityModule` | AuditLogger, AgentSandbox, Trust, Gate | Core security services |
| `ProofModule` | ProofSigning, ProofVerification | Liability proofs |
| `WebAuthnModule` | WebAuthn | Hardware authentication |
| `DnsModule` | DnsResolution | Trust lookup |
| `NexusModule` | Nexus | Protocol translation |
| `DashboardModule` | Dashboard controller | Admin UI endpoints |
| `EnterpriseModule` | Enterprise features | Licensed add-ons |

---

## 17. Complete File Map

### Services (10 total)

| File | Lines | Purpose |
|------|-------|---------|
| `trust.service.ts` | 606 | Long-term reputation |
| `agent-sandbox.service.ts` | 867 | Runtime safety |
| `webauthn.service.ts` | 422 | Hardware auth |
| `proof-signing.service.ts` | 105 | Create proofs |
| `proof-verification.service.ts` | 252 | Verify proofs |
| `dns-resolution.service.ts` | 271 | Trust lookup |
| `nexus.service.ts` | 262 | Protocol translation |
| `gate.service.ts` | ~150 | Rust bridge |
| `audit-logger.service.ts` | ~200 | Security logging |

### Controllers (7 total)

| File | Purpose |
|------|---------|
| `agents.controller.ts` | Agent CRUD |
| `proof.controller.ts` | Proof API |
| `webauthn.controller.ts` | Passkey flows |
| `dns.controller.ts` | Trust resolution API |
| `nexus.controller.ts` | Protocol translation API |
| `dashboard.controller.ts` | Admin endpoints |
| `csp-report.controller.ts` | Security policy violations |

### Entities (8 total)

| File | Purpose |
|------|---------|
| `agent-record.entity.ts` | Agent storage |
| `trust-record.entity.ts` | Trust scores & relationships |
| `trust-event.entity.ts` | Trust history |
| `audit-event.entity.ts` | Audit trail |
| `webauthn-credential.entity.ts` | Passkeys |
| `verification-key.entity.ts` | Public keys |
| `system-config.entity.ts` | System settings |

---

## Next Steps

After reading this document:
1. Try answering the [Comprehension Check](pillar_1_identity.md#6-comprehension-check-)
2. Explore the code with the file reference above
3. Move to [Gate Pillar](pillar_2_gate.md) (coming next)

---

*Last updated: 2025-12-31*

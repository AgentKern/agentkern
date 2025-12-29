# The Mantle Manifesto

## Why Every AI Agent Needs an Operating System

*December 2025*

---

> "AI agents are the new apps. But unlike apps, they can act autonomously. Without infrastructure, they become liabilities."

---

## The Problem We're Solving

The year is 2025, and AI agents are everywhere.

They browse the web, write code, make purchases, send emails, and interact with each other. OpenAI's Operator, Anthropic's Claude Computer Use, and dozens of open-source frameworks have made it trivial to build an agent that can *do things*.

But here's the problem nobody is talking about:

**There's no infrastructure for agent accountability, safety, memory, coordination, payments, or interoperability.**

When your agent makes a $50,000 purchase by mistake, who's liable? When two agents try to modify the same database record, who wins? When your agent drifts from its original goal, how do you detect it? When agents need to pay each other for services, how do they transact? When an AutoGen agent needs to talk to a Claude agent, what protocol do they use?

These are infrastructure problems. And they're unsolved.

---

## The Operating System Analogy

In the 1970s, every program managed its own memory, file access, and hardware. It was chaos. Then Unix came along and said: *"These are common problems. Let the OS handle them."*

AI agents are in the same position today. Every framework is reinventing:
- Authentication and identity
- Safety guardrails
- State management
- Resource coordination
- Payment rails
- Protocol translation

We believe it's time for an **Agentic Operating System**.

---

## The Physics of Trust (The Bedrock)

Software alone is not enough. You cannot have "Safety" if the admin can read the Agent's memory dump. You cannot have "Identity" if the private key is in a plain text file.

True sovereignty requires **Hardware Roots of Trust**.

AgentKern sits on a bedrock of **Confidential Computing (TEEs)**. Whether it's Intel TDX, AMD SEV-SNP, or AWS Nitro Enclaves, we bind the Agent's existence to silicon. Even we—the infrastructure providers—cannot see your Agent's thoughts.

**Six Pillars. One Bedrock.**

---

## Introducing AgentKern

**AgentKern** is the foundational layer for autonomous AI agents.

We provide six universal primitives — the **Six Pillars** — that every agent needs:

### 🪪 Identity
Every agent action is cryptographically signed. Private keys never leave the **Hardware Enclave (HSM/TEE)**. When something goes wrong, you know *exactly* which agent did what and when. Liability is traceable. Agents have **verifiable reputations** built on their transaction history via W3C Verifiable Credentials.

### 🛡️ Gate  
Before any action executes, it passes through our **Neuro-Symbolic Verification Engine**. Deterministic policy checks in under 1ms. Semantic malice detection (prompt injection, social engineering) in under 20ms. Multi-layer defense against the 73% of LLM apps that are vulnerable.

### 🧠 Synapse
Agents have goals, not just actions. Synapse tracks **Intent Paths** — the progression from goal to completion. When an agent starts taking more steps than expected, we detect **drift** before it becomes a problem. CRDTs enable global sync without coordination. Automatic alerting via webhooks.

### ⚖️ Arbiter
When multiple agents need the same resource, Arbiter provides **Atomic Business Locks** with priority-based scheduling. Raft consensus for strong consistency. Kill switch for emergency termination. No race conditions. No double-spending. No chaos.

### 💰 Treasury
Agents can **pay each other** for services. Treasury provides atomic transfers with 2-phase commit, spending budgets, micropayment aggregation, and **carbon footprint tracking**. The missing infrastructure for the **Agentic Economy**.

### 🔀 Nexus
The **universal protocol gateway**. Agents from different vendors — Google (A2A), Anthropic (MCP), W3C (ANP), ECMA (NLIP), NEAR (AITP) — can now talk to each other. Nexus provides protocol translation, agent discovery, task routing, and a **marketplace** for agent services.

---

## The Technical Stack

We're not building toys. AgentKern is engineered for production:

| Layer | Technology | Why |
|-------|------------|-----|
| SDK | TypeScript | Developer experience, ecosystem fit |
| Core | Rust | Performance, memory safety, zero GC |
| State | CRDTs | Eventual consistency without coordination |
| Consensus | Raft | Strong consistency when needed |
| Neural | ONNX | Fast ML inference (<20ms) |
| Sandbox | WASM | Nano-isolation for untrusted code |
| Protocols | A2A, MCP, ANP | Industry-standard agent communication |

Our verification engine processes **10,000+ requests per second** on a single node.

---

## Open Core Philosophy

AgentKern is **open source at the foundation**:

**Apache 2.0 Licensed (Free Forever)**:
- `@agentkern/sdk` — TypeScript client
- `agentkern-identity` — Agent authentication & trust
- `agentkern-gate` — Policy verification & prompt guard
- `agentkern-synapse` — State management & memory
- `agentkern-arbiter` — Coordination & compliance
- `agentkern-treasury` — Agent payments & carbon
- `agentkern-nexus` — Protocol gateway & marketplace

**Commercial (AgentKern Enterprise)**:
- Multi-node orchestration
- Global state synchronization  
- SAP, SWIFT, Mainframe connectors
- Compliance UI and audit logs
- Enterprise SLAs

We want AgentKern to be the *default* infrastructure for agents. The SDK will always be free.

---

## Global From Day One

AI agents don't respect borders. A US-built agent might process data for an EU customer stored in Singapore. Regulations like GDPR, PIPL, and DPDP create a compliance nightmare.

AgentKern handles this with **data residency awareness** built into every API:

```typescript
const client = new AgentKern({ region: 'eu' });
// All data stays in the EU. Cross-border transfers are blocked automatically.
```

And **sector-aware compliance** that adapts to local regulations:

| Sector | US | EU/MENA | AgentKern Action |
|--------|----|---------|--------------------|
| Finance | Interest-based | Takaful/Islamic | Switches from debt logic to pool logic |
| Health | HIPAA | GDPR | Switches data locality and consent flows |
| Commerce | Sales Tax | VAT | Switches tax calculation & invoicing |

---

## The Bigger Picture

We're not just building infrastructure. We're enabling a new economy.

With AgentKern, agents can:
- Have **verifiable reputations** built on their transaction history *(Identity + Trust Scoring)*
- **Pay each other** for services via micropayment rails *(Treasury)*
- **Talk to any other agent** regardless of vendor *(Nexus)*
- Track their **carbon footprint** for ESG compliance *(Treasury + Carbon Ledger)*
- Be **legally incorporated** as LLCs, shielding developers from liability
- Humans maintain **oversight and control** through escalation workflows

This is the **Agentic Economy**. And AgentKern is its operating system.

---

## Get Started Today

```bash
npm install @agentkern/sdk
```

```typescript
import { AgentKern } from '@agentkern/sdk';

const client = new AgentKern();
const agent = await client.identity.register('my-first-agent');

console.log('Welcome to the future.');
```

**Links:**
- [GitHub Repository](https://github.com/AgentKern/agentkern)
- [Documentation](https://github.com/AgentKern/agentkern/tree/main/docs)
- [Playground](https://github.com/AgentKern/agentkern/tree/main/apps/playground)

---

## Join Us

AgentKern is building the infrastructure layer for the next decade of AI. If you're excited about:

- Rust/TypeScript systems programming
- Distributed systems and CRDTs
- AI safety and alignment
- Protocol design (A2A, MCP)
- Developer experience

We want to hear from you.

**Star us on GitHub. Build with us. Shape the future of AI infrastructure.**

---

*The Mantle Foundation*  
*December 2025*

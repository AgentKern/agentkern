# Nexus Pillar Design

> **Universal Protocol Gateway** — The babel fish for autonomous agents.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Supported Protocols](#2-supported-protocols)
3. [Architecture](#3-architecture)
4. [Protocol Translation Engine](#4-protocol-translation-engine)
5. [Load Balancing & Routing](#5-load-balancing--routing)
6. [Agent Marketplace](#6-agent-marketplace)
7. [Chaos Proxy](#7-chaos-proxy)
8. [Google A2A Adapter](#8-google-a2a-adapter)
9. [Anthropic MCP Adapter](#9-anthropic-mcp-adapter)
10. [ECMA NLIP Adapter](#10-ecma-nlip-adapter)
11. [Complete Module Map](#11-complete-module-map)

---

## 1. Overview

**Nexus** is the interoperability layer of AgentKern. It allows agents running on different protocols (Google, Anthropic, Open Standards) to communicate seamlessly. It acts as a universal translator, load balancer, and marketplace.

### Core Mission

> "We do not copy; we innovate." — Nexus goes beyond simple bridging by providing a full **Protocol Translation Engine** that converts state, types, and capability negotiations on the fly.

---

## 2. Supported Protocols

Nexus supports the following standards (as of Dec 2025):

| Protocol | Developer | Features | Status |
|----------|-----------|----------|--------|
| **A2A** | Google | JSON-RPC, Multi-modal tasks | ✅ Stable |
| **MCP** | Anthropic | Tool use, Resource subscription | ✅ Stable |
| **NLIP** | ECMA | ECMA-430 Standard, Natural Language | 🟡 Beta |
| **ANP** | W3C | Agent Network Protocol | 🟡 Beta |
| **AITP** | NEAR | Crypto-native Agent Interop | 🟡 Beta |

---

## 3. Architecture

Nexus runs as a gateway service that sits between the external world and the internal AgentKern mesh.

```mermaid
graph TD
    Client[External Client] -->|HTTP/gRPC| Nexus
    
    subgraph Nexus [Nexus Gateway]
        Adapter[Protocol Adapters]
        Trans[Translator Engine]
        Router[Task Router]
        Consensus[Marketplace]
    end
    
    Nexus -->|Internal Msg| AgentA[Agent A (Native)]
    Nexus -->|Translated Msg| AgentB[Agent B (MCP)]
    Nexus -->|Translated Msg| AgentC[Agent C (A2A)]
```

---

## 4. Protocol Translation Engine

The **Translator** (`translator.rs`) is the heart of Nexus. It maps concepts between protocols.

### Capability Mapping

When an MCP agent talks to an A2A agent, Nexus translates:
- **MCP `tools/call`** $\rightarrow$ **A2A `tasks/create`**
- **A2A Artifacts** $\rightarrow$ **MCP Resources**
- **MCP Prompts** $\rightarrow$ **A2A Message Parts**

### Status Translation

Nexus normalizes task states across protocols using `TaskStatus`:

| Nexus State | A2A State | MCP State | NLIP State |
|-------------|-----------|-----------|------------|
| `Submitted` | `submitted` | `pending` | `init` |
| `Working` | `working` | `processing` | `active` |
| `Completed` | `completed` | `success` | `done` |
| `Failed` | `failed` | `error` | `error` |

---

## 5. Load Balancing & Routing

Managed by [`load_balancer.rs`](../../packages/pillars/nexus/src/router/load_balancer.rs).

### Strategies

1. **Round Robin**: Equal distribution.
2. **Least Connections**: Send to agent with fewest active tasks.
3. **Weighted**: Send more traffic to powerful agents (e.g., H100 vs T4).
4. **Random**: Stochastic distribution.
5. **Sticky**: Same client $\rightarrow$ Same agent (useful for conversational context).

### Health Checks

Nexus passively monitors agent health. If an agent fails a task or times out, it is marked `Unhealthy` and removed from rotation until it recovers.

---

## 6. Agent Marketplace

The **Marketplace** (`marketplace/mod.rs`) enables dynamic agent discovery and task bidding.

### Auction Flow

1. **Announcement**: User broadcasts a task (e.g., "Analyze this CSV").
2. **Bidding**: Agents submit `Bid`s with:
    - Price (`amount`)
    - Time (`estimated_time_secs`)
    - Confidence (`confidence` score)
3. **Evaluation**: Nexus calculates a `value_score` (lower is better).
    $$ \text{Score} = \text{Price} \times (1 + \text{TimeWeight}) / \text{Confidence} $$
4. **Award**: The best bid is selected, and funds are escrowed in **Treasury**.

---

## 7. Chaos Proxy

Located in [`chaos_proxy.rs`](../../packages/pillars/nexus/src/chaos_proxy.rs).

Simulates failures in external LLM providers (OpenAI, Anthropic) to test agent resilience.

- **Rate Limits** (429)
- **Downtime** (503)
- **High Latency** (timeout simulation)

Included locally to allow developers to build **Antifragile** agents without needing actual flaky internet checks.

---

## 8. Google A2A Adapter

Implementation: [`protocols/a2a.rs`](../../packages/pillars/nexus/src/protocols/a2a.rs)

- **Spec**: Google A2A v0.3 (July 2025)
- **Transport**: JSON-RPC over HTTP/SSE
- **Discovery**: `/.well-known/agent.json`

Handles the `A2AJsonRpcMessage` envelope and converts specific A2A methods (`tasks/send`, `agents/list`) into Nexus internal messages.

---

## 9. Anthropic MCP Adapter

Implementation: [`protocols/mcp.rs`](../../packages/pillars/nexus/src/protocols/mcp.rs)

- **Spec**: Model Context Protocol (June 2025)
- **Transport**: Stdio / SSE
- **Focus**: Connecting LLMs to Tools & Data

Supports:
- `tools/list`, `tools/call`
- `resources/read`, `resources/subscribe`
- `prompts/get`

---

## 10. ECMA NLIP Adapter

Implementation: [`protocols/nlip.rs`](../../packages/pillars/nexus/src/protocols/nlip.rs)

- **Spec**: ECMA-430 (Dec 2025)
- **Focus**: Natural Language Interaction

Uses `NLIPEnvelope` and supports `NLIPContent` types like `Text`, `Binary`, `Location`, and `Structured`. Designed for human-agent and agent-agent natural language capabilities.

---

## 11. AgentKern Native Adapter

Implementation: [`protocols/agentkern.rs`](../../packages/pillars/nexus/src/protocols/agentkern.rs)

The **Native** adapter handles direct internal communication between AgentKern nodes (VeriMantle nodes).

- **Spec**: AgentKern Native v1.0
- **Features**: 
    - Full fidelity `NexusMessage` transmission.
    - Zero-copy forwarding where possible.
    - Supported streaming.
    - **Extension Data**: Trust scores, Carbon footprint, Policy verification proofs.

---

## 12. Service Discovery

Implementation: [`discovery.rs`](../../packages/pillars/nexus/src/discovery.rs)

Nexus provides automated agent discovery using the `.well-known/agent.json` standard (A2A-compliant).

### Discovery Flow
1. **Fetch**: `GET https://agent-host.com/.well-known/agent.json`
2. **Parse**: Validate `AgentCard` schema (Capabilities, Endpoints, Keys).
3. **Register**: Add to `AgentRegistry` (`registry.rs`) for routing.
4. **Health Check**: Periodic `GET /health` polls to maintain routing table.

---

## 13. Complete Module Map

| Module | Lines | Purpose |
|--------|-------|---------|
| [`lib.rs`](../../packages/pillars/nexus/src/lib.rs) | 188 | Gateway entry point |
| [`protocols/a2a.rs`](../../packages/pillars/nexus/src/protocols/a2a.rs) | 238 | Google A2A Adapter |
| [`protocols/mcp.rs`](../../packages/pillars/nexus/src/protocols/mcp.rs) | 262 | Anthropic MCP Adapter |
| [`protocols/nlip.rs`](../../packages/pillars/nexus/src/protocols/nlip.rs) | 363 | ECMA-430 Adapter |
| [`protocols/agentkern.rs`](../../packages/pillars/nexus/src/protocols/agentkern.rs) | 65 | Native Adapter |
| [`protocols/translator.rs`](../../packages/pillars/nexus/src/protocols/translator.rs) | 291 | Translation Engine |
| [`router/load_balancer.rs`](../../packages/pillars/nexus/src/router/load_balancer.rs) | 366 | Traffic Routing |
| [`marketplace/mod.rs`](../../packages/pillars/nexus/src/marketplace/mod.rs) | 565 | Auction & Bidding |
| [`chaos_proxy.rs`](../../packages/pillars/nexus/src/chaos_proxy.rs) | 422 | LLM Failure Sim |
| [`agent_card.rs`](../../packages/pillars/nexus/src/agent_card.rs) | ~300 | Agent Metadata Schema |
| [`discovery.rs`](../../packages/pillars/nexus/src/discovery.rs) | 121 | Agent Discovery |
| [`registry.rs`](../../packages/pillars/nexus/src/registry.rs) | ~100 | In-memory Agent Registry |
| [`types.rs`](../../packages/pillars/nexus/src/types.rs) | ~200 | Core Types (NexusMessage, Task) |

**Total: ~3,500 lines of Rust**

---

*Last updated: 2025-12-31*

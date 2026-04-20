# Positioning: AgentKern vs Alternatives

This document defines AgentKern's intended position in the ecosystem and prevents category confusion.

## Category Definition

AgentKern is positioned as an **agent accountability kernel**:

- identity and trust anchors for agent actions
- policy and safety enforcement at decision boundaries
- coordination and arbitration for shared resources
- state/memory integrity controls
- compliance evidence and governance signals

It is **not** positioned as a full replacement for every orchestration framework, policy engine, or observability product.

## Where AgentKern Fits In The Stack

Typical production stack:

1. Orchestration/workflow framework (for planning, tool invocation, control flow)
2. Application services and business logic
3. AgentKern kernel layer (trust, policy, coordination, evidence)
4. Storage, messaging, and infrastructure

AgentKern can be embedded into existing stacks rather than requiring a full migration.

## Comparison By Concern

### Orchestration frameworks

Examples include graph/workflow and multi-agent orchestration platforms.

- Strong at: workflow composition, agent routing, developer ergonomics.
- AgentKern role: provide policy-grade and accountability-grade execution controls around those flows.

### Generic policy engines

Examples include policy decision engines and relationship-authorization systems.

- Strong at: broad policy modeling, authorization primitives.
- AgentKern role: apply policy semantics in agent-native contexts (intent, action risk, coordination outcomes).

### LLM/agent observability platforms

Examples include tracing and cost/latency monitoring tools.

- Strong at: telemetry, traces, debugging.
- AgentKern role: produce governance and compliance-relevant runtime signals and decision records.

## Build-vs-Buy Guidance

Use AgentKern when you need one or more of:

- verifiable accountability for autonomous or semi-autonomous actions
- deterministic controls for shared-resource contention
- enforceable policy boundaries for high-stakes operations
- compliance evidence workflows for regulated environments

Use lighter alternatives when:

- workload is single-agent and low-risk
- no regulated evidence requirements
- simple chat/tooling flows with no high-impact side effects

## Anti-Positioning (What Not To Claim)

- "AgentKern replaces all orchestration frameworks."
- "AgentKern eliminates the need for observability tooling."
- "AgentKern is a universal no-code platform."

These claims are strategically inaccurate and create avoidable buyer confusion.

## Product Message (Recommended)

Recommended short message:

> AgentKern is an open accountability kernel for multi-agent systems: identity, policy, coordination, memory integrity, and compliance evidence.

Recommended long message:

> AgentKern complements orchestration frameworks by enforcing trust, safety, and governance invariants at runtime. It is designed for teams operating autonomous workflows where accountability and control are mandatory.

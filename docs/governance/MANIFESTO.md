# AgentKern Manifesto (2026)

**"Bio-Digital Pragmatism: Advanced Runtimes, Not Magic"**

We build deterministic runtimes for autonomous AI agents. We reject "magic" metaphors in favor of hardware-linked safe execution.

---

## 1. The Hardware Root of Trust
Logic alone is insufficient for trust in agentic systems. Every agent action must be signed by a hardware-backed key.
- **Requirement**: Intel TDX or AMD SEV-SNP for encrypted memory.
- **Why**: If an action can't be traced to a physical enclave's measurement, it did not happen.

## 2. Symbolic Supremacy
Neural models are stochastic. Agents that handle assets or participate in regulated workflows must be governed by symbolic, rule-based policies (Gate) that fail-closed when intent is ambiguous.
- **Architecture**: **Neuro-Symbolic**. We combine the speed of Rust code (<1ms) with the intuition of small neural models (DistilBERT/ONNX <20ms) for intent capabilities.

## 3. Atomic Consistency
Multi-agent state is a distributed systems problem, not a "cognitive" one.
- **Technology**: **Conflict-free Replicated Data Types (CRDTs)** for memory and **2-Phase Commit** for transactions.
- **Principle**: There is no "vibe-based" consensus. State is mathematically consistent or the transaction aborts.

## 4. Sub-Millisecond Safety
Safety is irrelevant if it adds seconds of latency. We optimize safety enforcement at the kernel/runtime level.
- **Runtime**: Native **Tokio io_uring** for zero-copy I/O.
- **Target**: Safety checks must return in <10ms to enable real-time autonomous swarms.

## 5. Local Data Sovereignty
Agents must respect international data boundaries.
- **Implementation**: Native `DataRegion` enums (EU, US, CN, SA) enforced at the protocol level.
- **Rule**: PII never leaves its legal jurisdiction without explicit, signed consent.

---

## 🛠️ The Technology Bedrock

We choose boring, proven technologies over hype.

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **Language** | **Rust** | Memory safety without garbage collection pauses. |
| **Runtime** | **Tokio io_uring** | Maximum I/O throughput for high-concurrency swarms. |
| **Inference** | **ONNX Runtime** | Production-grade, cross-platform neural execution. |
| **Data** | **Polars / Arrow** | Columnar in-memory processing for adaptive queries. |
| **Actors** | **Actix** | Dynamic supervision and zero-downtime hot-swapping. |

> *"The most dangerous agents are the ones that do exactly what you asked. We build the runtime that ensures they do what you intended."*

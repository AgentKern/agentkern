# Technical Roadmap (2026)

This roadmap tracks **committed engineering work** for the next 2 quarters. All items list specific tracking issues.

## 🟢 Q1 2026: The Hardened Core

**Objective**: Reach 1.0 stability for the unified Rust server.

- [ ] **TEE Stabilization**: Promote `Intel TDX` support from experimental to stable.
- [ ] **Nexus Adapters**: Ship WASM adapters for `MCP` (Anthropic) and `Google A2A`.
- [ ] **Policy Grammar**: Formalize the DSL for Neuro-Symbolic verify checks.

## 🟡 Q2 2026: Multi-Node Mesh

**Objective**: Enable inter-node coordination without centralized bottlenecks.

- [ ] **Global CRDTs**: Implement `agentkern-synapse` delta-state replication.
- [ ] **Edge Handoff**: Stateless migration of agent context between data regions.

## ❌ Explicit Non-Goals (2026)

- **LLM Training**: We are a runtime, not a model lab.
- **Visual Programming**: We build infrastructure, not drag-and-drop tools.
- **Crypto Tokens**: We use verified ledgers, not volatile assets.

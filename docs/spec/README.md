# AgentKern Specifications (The Truth)

This directory contains the authoritative technical design specifications for the AgentKern Rust runtime. These documents define the "Source of Truth" for the implementation of the Six Pillars.

## 🏛️ The Six Pillars (All Rust)

| Pillar | Status | Specification |
|--------|--------|---------------|
| 🪪 **Identity** | ✅ Implemented | [IDENTITY_DESIGN.md](IDENTITY_DESIGN.md) |
| 🛡️ **Gate** | ✅ Implemented | [GATE_DESIGN.md](GATE_DESIGN.md) |
| 🧠 **Synapse** | ✅ Implemented | [SYNAPSE_DESIGN.md](SYNAPSE_DESIGN.md) |
| ⚖️ **Arbiter** | ✅ Implemented | [ARBITER_DESIGN.md](ARBITER_DESIGN.md) |
| 💰 **Treasury** | ✅ Implemented | [TREASURY_DESIGN.md](TREASURY_DESIGN.md) |
| 🔀 **Nexus** | ✅ Implemented | [NEXUS_DESIGN.md](NEXUS_DESIGN.md) |

---

## 🏗️ Support Systems

| Module | Purpose | Link |
| :--- | :--- | :--- |
| 📜 **Governance** | Regulatory Compliance as Code | [GOVERNANCE_DESIGN.md](GOVERNANCE_DESIGN.md) |
| 🛠️ **Infrastructure** | Unified Server & Edge Runtimes | [INFRASTRUCTURE_DESIGN.md](INFRASTRUCTURE_DESIGN.md) |
| 📦 **SDK Design** | Polyglot (Rust/Node/Python) Strategy | [SDK_DESIGN.md](SDK_DESIGN.md) |
| 🔌 **Integration** | Connecting Sidecars & External AI | [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md) |

---

## 🎨 Design Philosophy

1. **Rust-First**: All performance-critical logic and security boundaries are implemented in Rust.
2. **Deterministic**: We reject stochastic "reasoning" for security decisions.
3. **Local-Centric**: Identity and Safety are enforced at the edge/client level wherever possible.
4. **Pragmatic**: Specifications must match the current codebase. No visionary placeholders.

---

*Last updated: 2026-01-31*

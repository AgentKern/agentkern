# AgentKern Design Wiki

> **Purpose**: Learn the architecture through detailed design documentation.

---

## Pillars

| Pillar | Status | Document |
|--------|--------|----------|
| 🪪 **Identity** | ✅ Complete | [IDENTITY_DESIGN.md](IDENTITY_DESIGN.md) |
| 🛡️ **Gate** | ✅ Complete | [GATE_DESIGN.md](GATE_DESIGN.md) |
| 🧠 **Synapse** | ✅ Complete | [SYNAPSE_DESIGN.md](SYNAPSE_DESIGN.md) |
| ⚖️ **Arbiter** | ✅ Complete | [ARBITER_DESIGN.md](ARBITER_DESIGN.md) |
| 💰 **Treasury** | ✅ Complete | [TREASURY_DESIGN.md](TREASURY_DESIGN.md) |
| 🔀 **Nexus** | ✅ Complete | [NEXUS_DESIGN.md](NEXUS_DESIGN.md) |

### Foundation & Support

| Module | Status | Link |
| :--- | :--- | :--- |
| 📜 **Governance** | ✅ Complete | [GOVERNANCE_DESIGN.md](GOVERNANCE_DESIGN.md) |
| 🛠️ **Infrastructure** | ✅ Complete | [INFRASTRUCTURE_DESIGN.md](INFRASTRUCTURE_DESIGN.md) |
| 📦 **SDK Design** | ✅ Complete | [SDK_DESIGN.md](SDK_DESIGN.md) |

---

## Guides

| Guide | Description |
|-------|-------------|
| 🔌 **[Integration Guide](INTEGRATION_GUIDE.md)** | How to connect your agents to AgentKern |

---

## Operational

| Resource | Description |
| :--- | :--- |
| 🚨 **[Automated Recovery](runbooks/AUTOMATED_RECOVERY.md)** | Runbooks for autonomous mesh healing |
| 📊 **[Audit History](audits/)** | Historical security and compliance audits |

---

## Quick Reference

### The Six Pillars at a Glance

```
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│ Identity │ │   Gate   │ │ Synapse  │ │ Arbiter  │ │ Treasury │ │  Nexus   │
│    🪪    │ │    🛡️    │ │    🧠    │ │    ⚖️    │ │    💰    │ │    🔀    │
│ Passport │ │ Security │ │  Memory  │ │ Traffic  │ │   Bank   │ │ Network  │
└──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘
     TS          Rust         Rust         Rust         Rust         Rust
```

| Pillar | One-Sentence Summary |
|--------|---------------------|
| **Identity** | "Who is this agent? Can I trust them?" |
| **Gate** | "Is this action allowed? Is it safe?" |
| **Synapse** | "What was the original goal? Has the agent drifted?" |
| **Arbiter** | "Two agents want the same resource—who wins?" |
| **Treasury** | "How do agents pay each other? What's the spending limit?" |
| **Nexus** | "How do agents from different vendors communicate?" |

---

## Learning Path

Recommended order for learning the codebase:

1. **Start with Identity** (TypeScript) — Most approachable
2. **Then Gate** (Rust) — Core security, critical to understand
3. **Then Arbiter** — Coordination logic
4. **Then Synapse** — CRDTs are conceptually harder
5. **Then Treasury** — Payment logic
6. **Then Nexus** — Protocol handling

---

## Related Documentation

- [README.md](../README.md) — Project overview
- [ARCHITECTURE.md](../ARCHITECTURE.md) — System architecture
- [ENGINEERING_STANDARD.md](../ENGINEERING_STANDARD.md) — Code standards
- [SECURITY.md](../../SECURITY.md) — Security posture

---

*Last updated: 2025-12-31*

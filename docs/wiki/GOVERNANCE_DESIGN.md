# Governance & Compliance Design

> **Unified Compliance Framework** — Regulatory compliance as code, not paperwork.

---

## Table of Contents

1. [Overview](#1-overview)
2. [AI Governance (EU AI Act)](#2-ai-governance-eu-ai-act)
3. [Privacy & Data Sovereignty](#3-privacy--data-sovereignty)
4. [Audit Framework](#4-audit-framework)
5. [Complete Module Map](#5-complete-module-map)

---

## 1. Overview

**AgentKern Governance** is a library that embeds legal and regulatory requirements directly into the codebase. Agents cannot "forget" to comply because compliance is enforced at the compiler and runtime level.

### Core Philosophy

> "Compliance by Design" — We treat laws (GDPR, EU AI Act) as system constraints, similar to memory limits or CPU quotas.

---

## 2. AI Governance (EU AI Act)

Implementation: [`packages/governance/src/ai/eu_ai_act.rs`](../../packages/governance/src/ai/eu_ai_act.rs)

This module implements the **EU AI Act** (Aug 2025 enforcement) primitives.

### Risk Levels (Article 6)

The system classifies every agent interaction into one of four risk levels:

| Level | Description | Requirement |
|-------|-------------|-------------|
| **Prohibited** | Social scoring, manipulative AI | **HARD BLOCK** |
| **High Risk** | Biometrics, Critical Infra | Conformity Assessment, Logs |
| **Limited Risk** | Chatbots, Emotion Rec. | Transparency (Article 50) |
| **Minimal Risk** | Spam filters, Games | None |

### Automatic Technical Documentation (Article 11)

The `TechnicalDocumentation` struct automatically aggregates required evidence:

```rust
pub struct TechnicalDocumentation {
    pub system_description: SystemDescription,  // Art 11.1.a
    pub design_specs: DesignSpecifications,     // Art 11.1.b
    pub risk_management: RiskManagement,        // Art 9
    pub data_governance: DataGovernance,        // Art 10
    pub human_oversight: HumanOversight,        // Art 14
}
```

Agents can export this object to JSON to instantly generate a compliance artifact for auditors.

---

## 3. Privacy & Data Sovereignty

Implementation: [`packages/governance/src/privacy`](../../packages/governance/src/privacy)

### Global Privacy Registry

Handles region-specific data rules:
- **GDPR** (Europe): Right to be forgotten, Data residency.
- **CCPA** (California): Opt-out of sale.
- **LGPD** (Brazil), **PIPL** (China).

The `GlobalPrivacyRegistry` determines if data can leave a specific `CarbonRegion` or `SovereignZone`.

---

## 4. Audit Framework

Implementation: [`packages/governance/src/audit`](../../packages/governance/src/audit)

Provides a unified interface for **Evidence Collection**.

- **InfrastructureEvidence**: Collecting logs from TEEs.
- **ProcessEvidence**: Logs of human-in-the-loop approvals.
- **ModelEvidence**: Weights and biases versions.

---

## 5. Complete Module Map

| Module | Lines | Purpose |
|--------|-------|---------|
| [`ai/eu_ai_act.rs`](../../packages/governance/src/ai/eu_ai_act.rs) | 891 | EU AI Act implementation |
| [`ai/iso42001/`](../../packages/governance/src/ai/iso42001/) | ~200 | ISO 42001 AIMS Standard |
| [`privacy/global_registry.rs`](../../packages/governance/src/privacy/global_registry.rs) | ~300 | Data sovereignty rules |
| [`audit/mod.rs`](../../packages/governance/src/audit/mod.rs) | ~150 | Audit ledger types |
| [`industry/mod.rs`](../../packages/governance/src/industry/mod.rs) | ~100 | Industry verticals (HIPAA, etc.) |

**Total: ~1,600 lines of Rust**

---

*Last updated: 2025-12-31*

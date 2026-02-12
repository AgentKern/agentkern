# ADR-005: Feature Flag Standardization

**Status:** Accepted  
**Date:** 2026-01-28  
**Deciders:** AgentKern Engineering / Audit Remediation

---

## Context

The AgentKern codebase was found to have inconsistent feature flag usage across its Six Pillars:
- **Gate**: 11 features (complex interactions)
- **Synapse**: 5 features
- **Identity**: 4 features
- **Arbiter**: 3 features

This inconsistency makes it difficult for operators to understand which capabilities are enabled in a specific build and complicates CI/CD pipeline management.

## Decision

**Standardize feature flag naming and hierarchy across all pillars.**

The standardized naming scheme will follow:
1. `core`: Minimum functionality (enabled by default)
2. `standard`: Default production build
3. `enterprise`: Commercial features (EE)
4. `experimental`: Beta features

### Standard Flags Mapping

| Flag | Purpose | Status |
|------|---------|--------|
| `crypto-pqc` | Post-quantum cryptography | Standard |
| `geo-fence` | Data sovereignty controls | Standard |
| `neural` | Neural inference path | Standard |
| `esg` | Carbon tracking and vetoes | Standard |
| `ee-*` | All enterprise-only features | Enterprise |

## Rationale

### Industry research (2026)

| Finding | Source |
|---------|--------|
| "Feature flag bloat increases technical debt by 15%." | FinOps Alliance |
| "Consistent naming is the #1 factor in dev experience." | Rust Pulse Survey |
| "Hierarchical flags simplify Docker image tagging." | CloudCull Research |

## Consequences

### Positive
- Predictable build outcomes
- Simplified CI/CD matrices
- Easier cross-pillar capability detection (using `cfg(feature = "...")`)
- Clearer separation between OSS and Enterprise features

### Negative
- Initial refactoring effort required to rename existing flags
- Need to update documentation for custom builds

## Future Considerations

- Implement a global `agentkern-features` crate to centralize re-exports.
- Use `cargo-deny` to enforce that certain features are never enabled simultaneously (e.g., `mock` and `prod`).

---

## Implementation

**Planned Version:** 0.3.0  
**Affected Files:** `Cargo.toml` in all pillars, `apps/server/src/main.rs`.

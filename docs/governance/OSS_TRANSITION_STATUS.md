# OSS Transition Status

Last updated: 2026-04-19

This document tracks the repository's transition progress toward an OSS-first, production-ready posture.

## Objective

Establish a verifiable, low-ambiguity OSS operating model where:

- runtime behavior and public docs are consistent
- CI gates enforce quality/security contracts
- product boundary and positioning are explicit
- release risk is visible and governed

## Completed

- Added canonical OSS capability truth table:
  - `docs/OSS_CAPABILITY_MATRIX.md`
- Aligned user-facing docs to OSS runtime behavior:
  - `README.md`
  - `docs/OSS_SETUP.md`
  - `docs/reference/API_REFERENCE.md`
  - `docs/README.md`
- Added explicit runtime mode and pillar state contract:
  - startup mode logging in `apps/server/src/main.rs`
  - health payload fields:
    - `edition_mode`
    - `active_pillars`
    - `quarantined_pillars`
- Added runtime contract test:
  - `health_payload_contract_is_stable` in `apps/server/src/main.rs`
- Added OSS consistency automation:
  - `scripts/verify-oss-capability-consistency.sh`
  - wired into CI (`oss-capability-consistency` job)
- Hardened critical CI checks to blocking:
  - license compliance
  - dependency audit
  - docker build
  - release publish flow step behavior
- Added strategic clarity docs:
  - `docs/governance/OSS_PRODUCT_BOUNDARY.md`
  - `docs/reference/POSITIONING_VS_ALTERNATIVES.md`
  - `docs/governance/CI_POLICY.md`
- Added CI policy enforcement automation:
  - `scripts/verify-ci-policy.sh`
  - validates allowed advisory-only `continue-on-error` locations and comments

## Intentionally Advisory Checks

Advisory checks remain non-blocking by design:

- Coverage generation (`cargo tarpaulin`)
- SonarCloud scan

Policy and rationale:

- `docs/governance/CI_POLICY.md`

## Open Items

### 1) Release workflow publish completeness

Status: Completed  
Current state: release workflow now enforces deterministic publishability checks and explicit publish intent.

Exit criteria:

- [x] publish targets are explicitly defined
- [x] release flow is deterministic for tagged releases
- [x] failure behavior is documented

### 2) Operational SLO/SLA governance docs linkage

Status: In progress  
Current gap: CI and OSS boundary are defined, but explicit SLO/SLA governance linkage in release docs can be tightened.

Exit criteria:

- release docs reference required operational quality gates
- runbook links are explicit for incident handling

### 3) Contributor onboarding path compression

Status: In progress  
Current gap: docs are broad; a shortest-path “new maintainer checklist” is still useful.

Exit criteria:

- single quick path for first external contributor PR to pass all mandatory gates

## OSS Transition Release Gates (Suggested)

A transition milestone release is ready when all are true:

- [x] `cargo test --workspace` passes on main
- [x] `cargo test --bin agentkern-server` passes on main
- [x] `./scripts/verify-oss-capability-consistency.sh` passes
- [x] `./scripts/verify-ci-policy.sh` passes
- [x] no docs/runtime contradictions for route availability
- [x] release workflow publish behavior is finalized and documented
- [ ] advisory check outcomes reviewed and signed off by release owner

## Governance Rule

Any change that modifies runtime capability claims must update:

- `apps/server/src/main.rs` (runtime contract if needed)
- `docs/OSS_CAPABILITY_MATRIX.md`
- `README.md` and related setup/reference docs
- relevant tests and CI consistency scripts

## Notes

This document is a status tracker, not a strategy replacement.
Authoritative strategy and audit context:

- `docs/governance/OSS_AUDIT_AND_MARKET_RESEARCH_2026-04-19.md`
- `docs/governance/OSS_PRODUCT_BOUNDARY.md`
- `docs/governance/CI_POLICY.md`

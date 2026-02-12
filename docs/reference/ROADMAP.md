# Technical Roadmap (Next 90 Days)

This roadmap defines the execution plan for shipping a production-real `v0` of AgentKern.

## North Star

Ship one hardened golden flow:

`ingest -> gate -> arbiter -> treasury -> synapse -> decision`

with stable APIs, retries/idempotency, observability, and security controls.

## `v0` Exit Criteria (Day 90)

- [ ] Versioned service contracts are published and backwards-compatibility policy is documented.
- [ ] Golden flow succeeds in staging with failure injection and recovery validation.
- [ ] Security baseline is enforced (authn/authz, signed events, secrets policy, audit trail).
- [ ] SLO dashboards and alerting are active for core control-plane paths.
- [ ] SDK quickstart + sample app demonstrate end-to-end integration in under 15 minutes.

## Days 0-30: Foundations and Contracts

**Objective:** Freeze interfaces and remove delivery ambiguity.

- [ ] Define API/event contracts for `gate`, `arbiter`, `treasury`, `synapse`, and `identity`.
- [ ] Introduce explicit error taxonomy and idempotency keys across cross-pillar calls.
- [ ] Implement minimal persistence model for decisions, policy outcomes, and treasury reservations.
- [ ] Add contract tests and one smoke integration test for the golden path.
- [ ] Publish operational runbook skeletons (incident, rollback, and emergency policy freeze).

## Days 31-60: Reliability and Security Hardening

**Objective:** Make the core path resilient and production-safe.

- [ ] Implement retry strategy, dead-letter handling, and replay-safe processing.
- [ ] Add arbitration fairness checks and deterministic conflict-resolution tests.
- [ ] Enforce treasury guardrails (budget caps, reconciliation, and denial reason visibility).
- [ ] Add authn/authz middleware, service identity checks, and signed decision records.
- [ ] Instrument core paths with traces, RED metrics, and actionable alerts.

## Days 61-90: Productization and Pilot Readiness

**Objective:** Make AgentKern adoptable by external teams.

- [ ] Finalize SDK ergonomics and keep only stable surfaces in quickstart docs.
- [ ] Publish one canonical starter integration app (`Express` or `FastAPI` or `Axum`) with middleware examples.
- [ ] Deliver reference deployment profile (`docker-compose` + env contract validation).
- [ ] Run controlled pilot scenario and document integration friction points.
- [ ] Close top reliability gaps from pilot findings with targeted hardening.
- [ ] Tag `v0` release candidate and publish release notes + support matrix.

## Epic Breakdown

- [ ] **EPIC-1: Control-Plane Contracts**
  - Owners: architecture + platform
  - Scope: schemas, compatibility policy, integration contract tests
- [ ] **EPIC-2: Deterministic Gate + Arbiter**
  - Owners: policy + coordination
  - Scope: policy evaluation determinism, conflict resolution, explainability fields
- [ ] **EPIC-3: Treasury Guarantees**
  - Owners: treasury
  - Scope: reservation lifecycle, reconciliation, fail-closed behavior
- [ ] **EPIC-4: Synapse Consistency**
  - Owners: state/runtime
  - Scope: memory consistency, state replay safety, drift detection hooks
- [ ] **EPIC-5: Security and Governance**
  - Owners: security + governance
  - Scope: authn/authz, signed artifacts, auditability, policy control gates
- [ ] **EPIC-6: DX and Adoption**
  - Owners: sdk + docs
  - Scope: quickstart, examples, canonical starter app, reference configs, operator guide

## Immediate Sprint (Next 2 Weeks)

- [ ] Freeze first-pass API contracts for all five pillars.
- [ ] Land a full golden-path integration test with seeded fixtures.
- [ ] Add idempotency and typed error mapping on cross-pillar boundaries.
- [ ] Create baseline metrics dashboard for decision latency and failure rate.
- [ ] Publish a concise `v0` scope page and reject out-of-scope work by default.

## Explicit Non-Goals (v0 Window)

- [ ] Multi-region active-active orchestration.
- [ ] Visual workflow builder and low-code orchestration UX.
- [ ] New protocol adapters beyond immediate pilot requirements.
- [ ] Experimental research features without production SLO ownership.

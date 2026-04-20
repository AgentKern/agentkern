# OSS Product Boundary

This document defines the public OSS boundary for AgentKern so contributors, adopters, and enterprise buyers share the same expectations.

## Intent

The OSS boundary exists to:

- keep public runtime behavior explicit and testable
- prevent documentation drift
- reduce ambiguity between OSS defaults and enterprise overlay paths

## OSS Includes

In this repository, OSS includes:

- core crates under `packages/`
- applications under `apps/`
- SDKs under `sdks/`
- documentation under `docs/`

License baseline is Apache 2.0 for OSS repository artifacts.

## OSS Runtime Defaults (Unified Server)

Canonical source: `docs/OSS_CAPABILITY_MATRIX.md`

Default OSS server mode:

- active routes:
  - `/api/v1/identity/*`
  - `/api/v1/gate/*`
  - `/api/v1/synapse/*`
  - `/api/v1/arbiter/*`
  - `/api/v1/nexus/*`
- quarantined route:
  - `/api/v1/treasury/*`

Runtime contract is surfaced by:

- startup logs (`edition_mode`, active/quarantined pillars)
- `GET /health` payload fields:
  - `edition_mode`
  - `active_pillars`
  - `quarantined_pillars`
  - `pillars`

## Enterprise Overlay Boundary

Enterprise functionality is integrated via private overlay workflow (`ee/`).

Public repo must not:

- imply enterprise overlay behavior is active by default in OSS runtime
- advertise quarantined endpoints as available in OSS defaults

## Claim Governance Rules

Before merging any route/capability messaging change:

1. Update runtime wiring (if applicable)
2. Update `docs/OSS_CAPABILITY_MATRIX.md`
3. Update user-facing docs (`README.md`, `docs/OSS_SETUP.md`, API reference)
4. Pass CI consistency checks (`scripts/verify-oss-capability-consistency.sh`)

## Non-Negotiable Quality Gates

- Security and license checks are blocking in CI.
- Runtime capability claims must be verifiable in code and tests.
- Health contract changes require test updates.

## Revenue Compatibility Statement

A fully OSS core remains commercially viable through:

- managed control plane offerings
- enterprise SLAs and support
- compliance and operational assurance services
- certified deployment and incident-response services

This boundary document governs product truthfulness, not commercial strategy limits.

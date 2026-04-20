# OSS Capability Matrix

This document is the source of truth for what is available in public OSS builds of AgentKern.

## Build and Runtime Scope

- OSS repository license: Apache 2.0
- OSS runtime mode: core open-source mode
- Enterprise extensions: private overlay in `ee/` (not part of this repository)

## Pillar Availability (OSS Default)

| Pillar | Library in `packages/` | HTTP route in `agentkern-server` | OSS status |
|---|---|---|---|
| Identity | Yes | `/api/v1/identity/*` | Active |
| Gate | Yes | `/api/v1/gate/*` | Active |
| Synapse | Yes | `/api/v1/synapse/*` | Active |
| Arbiter | Yes | `/api/v1/arbiter/*` | Active |
| Nexus | Yes | `/api/v1/nexus/*` | Active |
| Treasury | Yes | `/api/v1/treasury/*` | Quarantined in OSS server |

## Health Contract

`GET /health` reports the canonical runtime status:

- `pillars.identity = active`
- `pillars.gate = active`
- `pillars.synapse = active`
- `pillars.arbiter = active`
- `pillars.nexus = active`
- `pillars.treasury = quarantined`

## Why Treasury Is Quarantined In OSS Server

The current unified server configuration reserves Treasury HTTP activation for enterprise overlay workflows.
The Treasury Rust crate remains part of the open repository, but the public server route is intentionally disabled in default OSS runtime wiring.

## Operational Guidance

- If your integration requires only active OSS routes, target:
  - `/api/v1/identity/*`
  - `/api/v1/gate/*`
  - `/api/v1/synapse/*`
  - `/api/v1/arbiter/*`
  - `/api/v1/nexus/*`
- Treat `/api/v1/treasury/*` as unavailable in OSS defaults.

## Change Management Rule

Any change to route availability in `apps/server/src/main.rs` must update:

- `docs/OSS_CAPABILITY_MATRIX.md`
- `docs/OSS_SETUP.md`
- `README.md` (if user-facing capability messaging changes)

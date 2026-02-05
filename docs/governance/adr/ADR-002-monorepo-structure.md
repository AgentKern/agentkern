# ADR-002: Monorepo Structure and Directory Boundaries

## Status
Accepted

## Date
2026-01-01

## Context

AgentKern uses a monorepo structure with multiple directories. The audit identified a need to explicitly document the boundaries between `apps/`, `packages/`, `ee/`, and `sdks/`.

## Decision

### Directory Structure

```
agentkern/
├── apps/              # Deployable applications (TypeScript)
│   └── identity/      # Identity service (NestJS)
│
├── packages/          # Shared libraries (Rust + TypeScript)
│   ├── pillars/       # The Six Core Pillars (Rust)
│   │   ├── gate/      # Security & Policy Enforcement
│   │   ├── synapse/   # State & Memory
│   │   ├── arbiter/   # Traffic Control & Governance
│   │   ├── treasury/  # Payments & Carbon
│   │   └── nexus/     # Protocol Gateway
│   │
│   └── foundation/    # Shared Infrastructure
│       ├── bridge/    # N-API binding (Rust → Node.js)
│       ├── runtime/   # WASM isolation layer
│       ├── governance/# EU AI Act compliance
│       └── parsers/   # Legacy protocol parsers
│
├── ee/                # Enterprise Edition (Rust)
│   └── sso/           # SAML/OIDC SSO
│
├── sdks/              # Client SDKs (all languages)
│   ├── typescript/    # TypeScript/Node.js SDK
│   ├── python/        # Python SDK (future)
│   ├── go/            # Go SDK (future)
│   └── java/          # Java SDK (future)
│
├── wasm-policies/     # Hot-swappable WASM policy modules
│   └── prompt-guard/  # Prompt injection detection
│
└── observability/     # Monitoring infrastructure
```

### Boundary Rules

| Directory | Language | Deploys As | Contains |
|-----------|----------|------------|----------|
| `apps/` | TypeScript | Container/Service | HTTP APIs, UI |
| `packages/pillars/` | Rust | Library (N-API) | Core business logic |
| `packages/foundation/` | Rust | Library | Shared utilities |
| `ee/` | Rust | Library | Licensed features |
| `sdks/` | Multi | npm/PyPI/etc. | Client libraries |
| `wasm-policies/` | Rust→WASM | Hot-swap modules | Policy logic |

### Decision Criteria

**When to add to `apps/`:**
- It's a standalone deployable service
- It has its own HTTP endpoints
- It's the entry point for users/agents

**When to add to `packages/pillars/`:**
- It implements one of the Six Pillars
- It's performance-critical (Rust)
- It's called via N-API from apps/

**When to add to `packages/foundation/`:**
- It's shared by multiple pillars
- It's infrastructure (runtime, bridge)

**When to add to `ee/`:**
- It requires an enterprise license
- It's not open-source

**When to add to `sdks/`:**
- It's a client library for external developers
- It wraps the AgentKern API

**When to add to `wasm-policies/`:**
- It's hot-swappable policy logic
- It runs in the WASM sandbox

## Consequences

### Positive
- Clear ownership boundaries
- Predictable code location
- Easier onboarding

### Negative
- Some duplication between pillars (acceptable for isolation)
- Must maintain multiple build systems (Cargo + pnpm)

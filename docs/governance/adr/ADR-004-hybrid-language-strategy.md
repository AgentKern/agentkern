# ADR-004: Hybrid Rust/TypeScript Language Strategy

## Status
Accepted

## Date
2026-01-01

## Context

AgentKern uses both Rust and TypeScript. The audit identified a need to document why Identity is in TypeScript while the Core Pillars are in Rust.

## Decision

### Language Assignment

| Component | Language | Rationale |
|-----------|----------|-----------|
| Identity (apps/identity) | TypeScript | Developer experience, ecosystem (NestJS, WebAuthn) |
| Gate, Synapse, Arbiter, Treasury, Nexus | Rust | Performance, safety, zero-copy |
| Bridge (N-API) | Rust | Links Rust to Node.js |
| SDKs | Multi | Developer reach |
| WASM Policies | Rust→WASM | Sandboxing, hot-swap |

### Why TypeScript for Identity?

1. **WebAuthn Ecosystem**: `@simplewebauthn/server` is the gold standard
2. **NestJS Productivity**: Enterprise patterns, decorators, DI
3. **Developer Onboarding**: Most contributors know TypeScript
4. **HTTP APIs**: Node.js excels at I/O-bound web services
5. **Rapid Iteration**: Business logic changes frequently

### Why Rust for Core Pillars?

1. **Zero-Copy Performance**: Agent guardrails run on every request
2. **Memory Safety**: No GC pauses, no data races
3. **WASM Target**: Policies compile to sandboxed WASM
4. **Cryptography**: audited crates (ring, ed25519-dalek)
5. **Predictable Latency**: No GC means microsecond SLAs

### N-API Bridge Strategy

```
┌─────────────────────────────────┐
│  apps/identity (TypeScript)     │
│  - WebAuthn, HTTP routing       │
└─────────────┬───────────────────┘
              │ N-API (0ms)
┌─────────────▼───────────────────┐
│  packages/foundation/bridge     │
│  - napi-rs bindings             │
└─────────────┬───────────────────┘
              │ Direct call
┌─────────────▼───────────────────┐
│  packages/pillars/* (Rust)      │
│  - Gate, Synapse, Arbiter, etc. │
└─────────────────────────────────┘
```

**Why N-API over gRPC/HTTP?**
- 0ms latency (direct function call)
- No serialization overhead for hot paths
- Guardrails need sub-millisecond response

### SDK Strategy (Addressing User Question)

**Current State:** SDKs in `sdks/typescript/agentkern` wrap the HTTP API.

**Future Vision:**
```
┌─────────────────────────────────────────────────────────────┐
│                       AgentKern API                          │
│  (apps/identity exposes HTTP endpoints for all Six Pillars)  │
└─────────────────────────────────────────────────────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          ▼                    ▼                    ▼
   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
   │  TypeScript  │     │    Python    │     │     Go       │
   │     SDK      │     │     SDK      │     │    SDK       │
   └──────────────┘     └──────────────┘     └──────────────┘
```

**SDK provides:**
- Type-safe API client
- Error handling (AgentKernError hierarchy)
- Retry logic with exponential backoff
- Middleware for frameworks (Express, NestJS)

**SDKs do NOT contain:**
- Core pillar logic (that's in Rust)
- Policy enforcement (that's via API)

**Multi-Language SDK Approach:**
1. **Generate from OpenAPI**: Auto-generate clients for Python, Go, Java, C#
2. **Language-Idiomatic**: Each SDK follows language conventions
3. **Thin Clients**: SDKs just wrap HTTP; logic is server-side

## Consequences

### Positive
- Best tool for each job
- TypeScript for I/O, Rust for CPU
- 0ms latency for security checks

### Negative
- Two build systems (Cargo + pnpm)
- N-API requires native compilation
- Contributors need both languages

### Mitigations
- CI builds for all platforms
- Clear documentation of when to use each
- Most work is in TypeScript (easier onboarding)

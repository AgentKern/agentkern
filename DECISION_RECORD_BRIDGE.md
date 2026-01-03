# Decision Record: Node.js ↔ Rust Bridge Strategy

**Status:** Approved  
**Date:** 2025-12-31  
**Revised:** 2025-12-31 (Hybrid Approach)

---

## Context

The `apps/identity` (Node.js) needs to call `packages/pillars/gate` (Rust) for policy enforcement.

We discovered that `packages/pillars/gate` has **two implementations**:
1. **N-API Bridge** (`packages/foundation/bridge/`) - Embedded library
2. **HTTP Server** (`packages/pillars/gate/src/bin/server.rs`) - Standalone microservice

---

## Decision: **HYBRID APPROACH** ✅

**Keep both. They serve different purposes.**

### Hot Path (N-API) - 0ms Latency
```
apps/identity → packages/foundation/bridge (N-API) → gate logic
```
- Prompt injection guard (every LLM call)
- Request validation (every request)
- TEE attestation (critical path)

### Cold Path (HTTP) - 1-5ms Latency
```
Admin UI → HTTP → packages/pillars/gate server (port 3001)
```
- Policy CRUD (occasional)
- Admin operations
- External integrations
- Multi-node policy sync

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     apps/identity                           │
│                                                             │
│  ┌──────────────────────────────────────┐                  │
│  │ "Hot Path" (every request)           │                  │
│  │ N-API Bridge → gate (embedded, 0ms)  │                  │
│  │ • Prompt guard                       │                  │
│  │ • Request validation                 │                  │
│  └──────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│               packages/pillars/gate (HTTP server)                   │
│               Port 3001 (optional container)                │
│                                                             │
│  ┌──────────────────────────────────────┐                  │
│  │ "Cold Path" (management)             │                  │
│  │ HTTP Server (Axum)                   │                  │
│  │ • Policy CRUD                        │                  │
│  │ • Admin operations                   │                  │
│  └──────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Files

| Component | Path | Purpose |
|-----------|------|---------|
| N-API Bridge | `packages/foundation/bridge/` | Hot path, 0ms latency |
| Gate Server | `packages/pillars/gate/src/bin/server.rs` | Cold path, HTTP management |
| Dockerfile | `packages/pillars/gate/Dockerfile` | Containerized standalone deployment |

---

## When to Use Which

| Use Case | Approach | Latency |
|----------|----------|---------|
| Prompt guard (every LLM call) | N-API | 0ms |
| Request validation | N-API | 0ms |
| Policy CRUD (admin) | HTTP | 1-5ms |
| External integrations | HTTP | Variable |
| Multi-node sync | HTTP | Variable |

---

## Rationale

1. **Performance**: Guardrails are blocking operations on the hot path. 0ms latency is critical.
2. **Flexibility**: HTTP server enables admin UIs, external integrations, independent scaling.
3. **Future-proof**: Can split to full microservices if scaling demands.

---

## Related

- [ADR-001: Gateway-Identity Merge](./adr/ADR-001-gateway-identity-merge.md)
- [ARCHITECTURE.md](./ARCHITECTURE.md)

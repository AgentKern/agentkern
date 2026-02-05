# ADR-001: Merge Gateway into Identity

**Status:** Accepted  
**Date:** 2025-12-31  
**Deciders:** AgentKern Engineering  

---

## Context

AgentKern had two TypeScript applications:
- `apps/gateway` - HTTP router for the Six Pillars
- `apps/identity` - Identity, WebAuthn, trust management

This created architectural inconsistency:
- Gateway embedded Rust Gate via N-API (0ms latency)
- Gateway proxied to Identity via HTTP (adds latency)
- Gateway contained mostly thin wrappers over Rust packages

## Decision

**Merge `apps/gateway` into `apps/identity`.**

The consolidated architecture:
```
apps/identity (single TypeScript entry point)
├── Identity/WebAuthn (original)
├── Nexus (protocol translation, merged from Gateway)
├── Enterprise licensing
└── Health endpoints
         │ N-API
packages/foundation/bridge → Rust crates (gate, arbiter, synapse)
```

## Rationale

### Industry Research (2025)

| Finding | Source |
|---------|--------|
| N-API: 0ms latency for CPU-bound tasks | triton.one |
| "Start monolith, evolve to microservices" | AI Agent Insider |
| Guardrails need sub-ms latency | LangChain |
| Zero Trust essential for AI agents | TechNewsWorld |

### Performance

| Approach | Latency | Coupling |
|----------|---------|----------|
| N-API (chosen) | 0ms | Tight, but justified |
| gRPC Service | 1-5ms | Loose |
| HTTP Proxy | 5-20ms | Loose |

For guardrails that run on every request, 0ms latency is critical.

### Simplicity

| Before | After |
|--------|-------|
| 2 TypeScript apps | 1 TypeScript app |
| 2 deployment units | 1 deployment unit |
| HTTP + N-API | N-API only |

## Consequences

### Positive
- Single deployment unit (simpler ops)
- 0ms latency for guardrails
- Cleaner codebase (-2,478 lines)

### Negative
- Cannot scale Nexus independently (for now)
- Identity app has more responsibility

### Mitigations
- NexusModule is modular; can split out later
- Rust packages handle heavy lifting
- Enterprise features go to ee/ directory

## Future Considerations

When to reconsider this decision:
1. **Nexus needs independent scaling** (>10K agents)
2. **Team grows** (need separate ownership)
3. **Multi-region** (deploy Nexus closer to agents)

At that point, extract NexusModule to a separate service with gRPC.

---

## Implementation

**Commit:** `bfad894`  
**Files changed:** 39 (291 insertions, 2,478 deletions)

### Moved to Identity
- `nexus.controller.ts` - Protocol translation
- `nexus.service.ts` - Agent registry
- `nexus.module.ts` - Module wiring
- `nexus.dto.ts` - DTOs

### Deleted
- All other Gateway files (handled by Rust via N-API)

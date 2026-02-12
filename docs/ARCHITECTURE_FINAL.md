# AgentKern Architecture (Final)

**Date**: 2026-01-03  
**Status**: ✅ Clarified and Documented

---

## 🎯 Architecture Overview

### The Six Pillars (All Rust Libraries)

```
packages/pillars/
├── identity/    🪪  Identity & Authentication
├── gate/        🛡️  Security & Policy Enforcement
├── synapse/     🧠  Memory & State Management
├── arbiter/     ⚖️  Coordination & Governance
├── treasury/    💰  Payments & Carbon Tracking
└── nexus/       🔀  Protocol Translation
```

**All implemented in Rust as libraries/crates.**

---

### The Unified Server (`apps/server/`)

**Purpose**: HTTP Gateway that routes to all pillars

**What it does**:
- Exposes all six pillars via HTTP REST API
- Routes `/api/v1/identity` → `packages/pillars/identity`
- Routes `/api/v1/gate` → `packages/pillars/gate`
- Routes `/api/v1/synapse` → `packages/pillars/synapse`
- Routes `/api/v1/arbiter` → `packages/pillars/arbiter`
- Routes `/api/v1/nexus` → `packages/pillars/nexus`
- Routes `/api/v1/treasury` → `packages/pillars/treasury`
- Provides JWT authentication (`/api/v1/auth`)
- Enterprise features (`/api/v1/ee`)

**It is NOT**:
- ❌ A pillar (pillars are in `packages/pillars/*`)
- ❌ The identity pillar (identity is a separate pillar)
- ❌ A replacement for identity (identity is a pillar, server is a gateway)

**It IS**:
- ✅ A unified HTTP gateway/router
- ✅ Single entry point for all SDKs
- ✅ Provides authentication, CORS, middleware

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│         apps/server (Rust - HTTP Gateway)                │
│                                                          │
│  HTTP Routes:                                            │
│  /api/v1/identity  → packages/pillars/identity          │
│  /api/v1/gate      → packages/pillars/gate              │
│  /api/v1/synapse   → packages/pillars/synapse           │
│  /api/v1/arbiter   → packages/pillars/arbiter           │
│  /api/v1/nexus     → packages/pillars/nexus             │
│  /api/v1/treasury  → packages/pillars/treasury          │
│  /api/v1/auth      → JWT authentication                 │
└──────────────────────┬──────────────────────────────────┘
                       │ HTTP/REST API
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   ┌─────────┐   ┌─────────┐   ┌─────────┐
   │TypeScript│   │ Python │   │   Go    │
   │   SDK    │   │   SDK   │   │   SDK   │
   └─────────┘   └─────────┘   └─────────┘
```

---

## Key Principles

1. **All Logic in Rust**: Six pillars are Rust libraries
2. **HTTP Gateway**: Server exposes pillars via REST API
3. **SDKs are HTTP Clients**: SDKs consume the server API contracts
4. **Single Entry Point**: One server, all pillars accessible

---

## SDK Strategy

### HTTP-Based SDKs

**Runtime Model**:
- All pillars are Rust (no TypeScript→Rust calls)
- Server is Rust (no TypeScript server)
- SDKs use HTTP API contracts

**SDK Implementation**:
1. Generate from OpenAPI/Swagger spec
2. Language-idiomatic HTTP clients
3. Thin clients (logic is server-side in Rust)

---

## Benefits

1. ✅ **Single Source of Truth**: All logic in Rust
2. ✅ **Language Agnostic**: SDKs in any language
3. ✅ **Simple Architecture**: HTTP API only
4. ✅ **Operational Simplicity**: One unified runtime path
5. ✅ **Easy SDK Generation**: Auto-generate from OpenAPI

---

## Status

✅ **Architecture Clarified**:
- 6 Pillars = Rust libraries
- Server = HTTP gateway
- SDKs = HTTP clients
- HTTP-native integration model

---

**Last Updated**: 2026-02-07

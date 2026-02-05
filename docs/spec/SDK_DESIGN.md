# SDK Architecture: Unified Rust Core (2026)

AgentKern uses a "Unified Core" strategy for its SDKs. Instead of reimplementing cryptographic and protocol logic in every language, we maintain a single source of truth in Rust (`agentkern-sdk-core`) and bind it to other languages.

## 🏗️ The Hybrid Architecture

```mermaid
graph TD
    subgraph "Rust Core (packages/sdk-core)"
        RC[Core Logic] --> Crypto[Ed25519 / PQC]
        RC --> Protocol[A2A / Nexus / MCP]
        RC --> Validation[Safety Verification]
    end

    subgraph "Bindings Layer"
        RC --> |napi-rs| TS[TypeScript / Node.js]
        RC --> |uniffi| PY[Python]
        RC --> |uniffi| CS[C# / .NET]
    end

    subgraph "Consumers"
        TS --> |npm| SDK_TS[@agentkern/sdk]
        PY --> |pip| SDK_PY[agentkern]
    end
```

## 🛠️ Implementation Details

### 1. TypeScript SDK (`sdks/typescript`)
The Node.js SDK uses `napi-rs` to bind the Rust core. This allows sub-millisecond local proof generation and verification directly in the Node event loop without the overhead of HTTP calls to a sidecar.

- **Primary Interface**: `@agentkern/sdk`
- **Key Class**: `Agent` (High-level wrapper around the FFI handles)

### 2. Python SDK (`sdks/python`)
Targeted at data scientists and AI agent developers. Uses `UniFFI` to generate native Python bindings.

- **Integrations**: Native adapters for LangChain, CrewAI, and OpenAI Client.

---

## 🔒 Security Principles

1. **Local Signing**: Private keys never leave the agent's memory. Signing happens in the Rust core.
2. **Deterministic Handshakes**: All SDKs use the same state machine for A2A handshakes, ensuring perfect interoperability.
3. **Rust Integrity**: The core logic is compile-time verified for memory safety, protecting agents from common FFI buffer overflows.

## 📅 Roadmap (2026)

- **Q1**: Finalize N-API bindings for Identity management [DONE].
- **Q2**: UniFFI generation for Python and C# [IN PROGRESS].
- **Q3**: WASM Component Model (WCM) support for browser-based agents.

# AgentKern Polyglot SDK Architecture

## 1. The Challenge & Evolution of ADR-004
**ADR-004 (2026-01-01)** proposed generating SDKs from OpenAPI.
While excellent for HTTP Transport, pure OpenAPI clients **cannot handle Local Cryptography** (Signing/Attestation) efficiently across 5 languages without reimplementing the crypto stack 5 times.

**The Evolution: "Smart Clients"**
To support **Agent Sovereignty** (where the Agent holds the Private Key, not the Server), the SDK must do more than just HTTP. It needs a shared **Rust Core** for:
1.  **Local Signing** (Ed25519)
2.  **Protocol Handshake** (A2A encryption)
3.  **Governance Validation** (pre-flight checks)

Thus, we are upgrading the strategy from **Active Record (OpenAPI)** to **Data Mapper (Rust Core)**.

## 2. 2026 Research Analysis (UniFFI vs WASM)
Research into the "State of the Art 2026" highlights two competing standards for polyglot SDKs:

 | Feature | UniFFI (Universal FFI) | WASM Component Model (WCM) |
 |---------|------------------------|----------------------------|
 | **Maturity** | **High** (Production Ready) | **Medium** (Standardizing) |
 | **Use Case** | Native Mobile, Python, C++ | Serverless, Edge, Browser |
 | **Performance**| Native Speed (Zero-Cost) | Near-Native (JIT overhead) |
 | **Ecosystem** | Strong (Mozilla, Firefox) | Strong (Cloud Native, W3C) |

**Strategic Decision**:
We will adopt a **Hybrid Pragmatic Approach**:
1.  **UniFFI** as the primary driver for immediate Mobile/Python/C# support (Stable).
2.  **N-API (`napi-rs`)** for Node.js/Identity to ensure maximum performance for the backend.
3.  **WASM Compatible Core**: The Rust core will be written to be WASM-compatible, allowing us to pivot to WCM for "Plugin Agents" in the future.

## 3. The Solution: `sdk-core` + UniFFI

We will implement the "heavy lifting" once in Rust and generate native bindings for other languages.

### Architecture

```mermaid
graph TD
    subgraph "Core (Rust)"
        RC[packages/sdk-core] --> Tests
        RC --> Crypto[Signatures / Keys]
        RC --> Protocol[A2A / MCP Parsing]
        RC --> Types[Agent Models / DTOs]
    end

    subgraph "Bindings (Generated)"
        RC --> |uniffi-bindgen| Py[Python Wheel]
        RC --> |uniffi-bindgen| CS[C# DLL]
        RC --> |uniffi-bindgen| KT[Kotlin/Java JAR]
        RC --> |uniffi-bindgen| Swift[Swift Package]
        RC --> |napi-rs| Node[Node.js / TypeScript]
    end

    subgraph "Consumers"
        AgentPy[Python Agent (e.g., LangChain)] --> Py
        AgentNET[Enterprise App (.NET)] --> CS
        Identity[apps/identity (NestJS)] --> Node
    end
```

### 3. Identity Service Integration (`apps/identity`)
Since `apps/identity` is written in TypeScript (NestJS):
- It will **NOT** rewrite core logic.
- It will consume the **Node.js Integration** via `packages/foundation/bridge` (or the new `sdk-core` N-API binding).
- This ensures that the *server* (Identity) and the *clients* (Agents) use **exactly the same validation logic**.

### 5. OpenAI & LLM Integration
The SDK will provide "Middleware Adapters" for common AI frameworks.

**Example (Python):**
```python
import openai
from agentkern import AgentKern

# Initialize AgentKern SDK
kern = AgentKern(private_key="...")

# Wraps OpenAI client to automatically:
# 1. Sign requests
# 2. Inject trust headers
# 3. Verify tool outputs
client = kern.wrap_openai(openai.Client())
```

## 6. Roadmap

### Phase 1: Core & Node.js (High Priority)
- Create `packages/sdk-core` (Rust).
- Implement `Agent` struct, `register`, `sign`.
- Generate Node.js bindings (N-API).
- **Deliverable**: `@agentkern/sdk` (npm).

### Phase 2: Python (Data Science / AI)
- Generate Python bindings (UniFFI).
- Create `agentkern` (PyPI).
- Add OpenAI/LangChain adapters.

### Phase 3: Enterprise (C# / Java)
- Generate C# bindings for .NET.
- Generate Java bindings.

## 7. Directory Structure
```
packages/
  sdk-core/          # Rust library (THE TRUTH)
    src/lib.rs       # Core logic
    Cargo.toml
  
  sdk-node/          # N-API bindings
    index.ts         # TypeScript wrapper
    
sdks/
  python/            # Python package wrapper
  csharp/            # .NET solution
```

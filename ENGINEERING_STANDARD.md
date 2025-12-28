# The AgentKern Engineering Standard (2026)

**"Bio-Digital Pragmatism: Advanced Runtimes, Not Magic"**

We acknowledge that "Self-Rewiring AI" is dangerous.
Instead, we build **Adaptive Systems** using proven, high-performance technologies (Rust, WASM, ONNX).

---

## 0. The Hardware Root of Trust (The Bedrock)

Before software runs, hardware must be trusted. AgentKern Enterprise Nodes require:

| Component | Requirement | Why? |
|-----------|-------------|------|
| **CPU** | AMD EPYC (Zen 3+) or Intel Xeon (Sapphire Rapids+) | Must support **SEV-SNP** or **TDX** for encrypted memory. |
| **TPM** | TPM 2.0 | Secure boot and remote attestation. |
| **HSM** | FIPS 140-2 Level 3 | (Optional) High-assurance key storage for Identity pillars. |

---

## Technology Stack (December 2025 Research)

> [!NOTE]
> All technology choices below are based on comprehensive research conducted December 2025.
> They prioritize **production-readiness**, **ecosystem compatibility**, and **future-proofing**.

### Runtime: Native Tokio io_uring

| Alternative | Status | AgentKern Choice |
|-------------|--------|-------------------|
| **tokio-uring** | v0.5.0, io_uring being merged into Tokio core | ✅ **Selected** |
| glommio | Datadog-proven, thread-per-core | ❌ Separate ecosystem |
| monoio | ByteDance, highest perf | ❌ Requires nightly Rust |

**Rationale**: Tokio 1.48.0+ is actively integrating io_uring into core (Dec 2025 commits for `fs::read`).
Since AgentKern is built on Tokio (Axum, tracing), native tokio-uring ensures ecosystem compatibility
and long-term stability as features stabilize.

### Neural Inference: ONNX Runtime (ort)

| Version | Status |
|---------|--------|
| `ort v2.0.0-rc.10` | Production-ready, stable 2.0.0 expected Q1 2026 |
| ONNX Runtime 1.23.x | Current stable (1.24 releasing Jan 2026) |

**Use Cases**: Semantic code search, embeddings, TTS, neuro-symbolic guards.

### Data Processing: Polars + Arrow

| Library | Version | Status |
|---------|---------|--------|
| Polars | 1.36.1 | Released Dec 9, 2025 |
| Arrow Rust | v57.0.0 | Released Oct 30, 2025 (4x faster Parquet parsing) |

**Case Study**: Decathlon replaced Apache Spark with Polars — 4x faster startup, runs on single Kubernetes pod.

### Actor Framework: Actix

| Version | Rust Support |
|---------|--------------|
| Actix 0.13 | Stable Rust 1.68+ |

**Use**: Dynamic supervision with WASM hot-swapping for zero-downtime evolution.

---

## 1. The Macro-Architecture: "Dynamic Supervision" (Bio-Mimicry)
Instead of "Magic Mitosis," we use **Actor-Based Supervision w/ Hot-Swapping**.

*   **Technology**: **Rust Actors (Tokio/Actix) + WASM Component Model**.
*   **Implementation**: `packages/gate/src/actors.rs`
*   **Mechanism**:
    *   The "Cell" is a Supervisor Actor (`GateSupervisor`).
    *   The "Logic" is a **WASM Component** (`wasm/mod.rs`).
    *   **Innovation**: When logic needs to change (e.g., a new security patch), we **Hot-Swap the WASM Component** at runtime without dropping connections.
*   **Result**: Zero-downtime evolution. The "organism" heals its cells (replaces code) while running.

## 2. The Micro-Architecture: "Adaptive Execution" (Not Self-Writing Code)
We reject "AI writing code at runtime" (Hallucination Risk).

*   **Pattern**: **Adaptive Query Execution**.
*   **Technology**: **Rust + Arrow/Polars**.
*   **Implementation**: `packages/synapse/src/adaptive.rs`
*   **Mechanism**:
    *   The system monitors query performance (latency/throughput).
    *   It maintains **multiple execution plans**:
        - `Standard`: Safe, predictable
        - `Vectorized`: SIMD-optimized for low CPU pressure
        - `Streaming`: Out-of-memory datasets
    *   **Innovation**: The runtime switches execution strategies *per request* based on live system pressure.
*   **Result**: The system "optimizes itself" deterministically, not stochastically.

## 3. The Logic Core: "Neuro-Symbolic Guards" (The Neural Kernel)
We reject "LLM-as-OS" (Too slow/unpredictable).

*   **Pattern**: **Neuro-Symbolic Architecture**.
*   **Technology**: **Rust `ort` (ONNX Runtime) + DistilBERT/TinyLlama**.
*   **Implementation**: `packages/gate/src/neural/mod.rs`
*   **Mechanism**:
    1.  **Fast Path (Symbolic)**: Deterministic Code Checks (Policy, Signature). **<1ms**.
    2.  **Safety Path (Neural)**: A small, embedded model runs *alongside* to score "Semantic Malice" (e.g., social engineering attempts). **<20ms**.
*   **Innovation**: We combine the *speed* of code with the *intuition* of AI.

---

## 4. The Runtime Core: "The Hyper-Loop" (Zero-Copy I/O)

*   **Pattern**: **io_uring-based async runtime**.
*   **Technology**: **Native Tokio io_uring** (tokio-uring 0.5+).
*   **Implementation**: `packages/gate/src/runtime.rs`
*   **Mechanism**:
    *   Zero-copy file and network I/O on Linux 5.10+
    *   Falls back to standard Tokio on other platforms
    *   Thread-per-core architecture for predictable latency
*   **Why tokio-uring over alternatives**:
    *   Tokio is actively merging io_uring into core (Dec 2025)
    *   No ecosystem friction with Axum, Hyper, Tonic
    *   Will stabilize as Tokio evolves

---

## Summary of the "Pragmatic Mantle"

| Concept | The Hype (Theoretical) | The AgentKern Reality (Buildable) |
| :--- | :--- | :--- |
| **Topology** | AI Rewiring Infrastructure | **WASM Hot-Swapping** (Actix Supervision) |
| **Optimization** | LLM Writing Code | **Adaptive Query Execution** (Polars) |
| **Security** | LLM-as-Kernel | **Neuro-Symbolic** (Rust + ONNX) |
| **I/O** | Custom Runtime | **Native Tokio io_uring** |
| **Philosophy** | "Magic Organism" | **"Adaptive Machine"** |

---

## Supply Chain Security: "Artifacts, Not Just Code"

We don't just ship binaries; we ship **Proofs**.

*   **Reproducible Builds**: All release binaries must be byte-for-byte reproducible from source.
*   **SBOM (Software Bill of Materials)**: Every release includes a CycloneDX SBOM listing all dependencies.
*   **Sigstore Signing**: All artifacts are signed keylessly via Sigstore, tied to our OIDC identities.
*   **Provenance**: SLSA Level 3 compliance tracks the build environment and parameters.

---

## Feature Flags

```toml
# packages/gate/Cargo.toml
[features]
io_uring = ["tokio-uring"]   # Native Tokio io_uring
wasm = ["wasmtime"]          # WASM policy isolation
neural = ["ort"]             # ONNX neuro-symbolic
actors = ["actix"]           # Dynamic supervision
full = ["io_uring", "wasm", "neural", "actors"]

# packages/synapse/Cargo.toml  
[features]
adaptive = ["polars", "arrow"]  # Adaptive Query Execution
crdts = ["crdts-lib"]           # Conflict-free Replicated Data Types
graph = ["petgraph"]            # Graph Vector Database
```

---

## 5. Data Sovereignty: Hybrid DataRegion Model

Per December 2025 research on AWS/Azure regional strategies:

```rust
pub enum DataRegion {
    // Tier 1: Major Regulatory Blocs (strict localization)
    Us,      // HIPAA, CCPA, SOX
    Eu,      // GDPR, EU Data Act 2025
    Cn,      // PIPL (in-country required)
    
    // Tier 2: Emerging Sovereignty Blocs
    Mena,    // GCC Vision 2030, Saudi PDPL, Islamic Finance
    India,   // DPDP Act 2023
    Brazil,  // LGPD
    
    // Tier 3: Regional Fallbacks
    AsiaPac, // Singapore PDPA, Japan APPI, Australia
    Africa,  // Varying by country
    Global,  // Default
}
```

**Helper Methods**:
- `requires_localization()` → `true` for Cn, Eu, India, Mena
- `privacy_law()` → Returns applicable law name

**Implementation**: `packages/gate/src/types.rs`

---

## 6. Islamic Finance: Shariah Compliance Module

AgentKern provides comprehensive Islamic finance validation through the `shariah_compliance` module.

**Implementation**: `packages/gate/src/shariah_compliance.rs`

### Supported Transaction Types (10 Total)

| Type | Description | Shariah-Compliant |
|------|-------------|-------------------|
| **Takaful** | Islamic insurance (mutual risk pooling) | ✅ |
| **Murabaha** | Cost-plus financing | ✅ |
| **Musharakah** | Profit/loss partnership | ✅ |
| **Ijara** | Leasing | ✅ |
| **Sukuk** | Islamic bonds (asset-backed securities) | ✅ |
| **Wakala** | Agency contract | ✅ |
| **Salam** | Forward sale (advance payment) | ✅ |
| Trade | General trade | ✅ |
| Insurance | Conventional insurance | ❌ |
| Loan | Interest-based loan | ❌ |

### Validation Rules

| Principle | Detection | Penalty |
|-----------|-----------|---------|
| **Riba** (Interest) | `interest_rate > 0` | -50 score, non-compliant |
| **Gharar** (Uncertainty) | No underlying asset | -20 to -40 score |
| **Maysir** (Gambling) | Guaranteed outcome on insurance | -30 score |

### Usage

```rust
use agentkern_gate::shariah_compliance::{ShariahComplianceValidator, TransactionType};

let validator = ShariahComplianceValidator::new();
let result = validator.validate(&transaction)?;

if result.compliant {
    // Proceed with Shariah-compliant transaction
} else {
    // Review recommendations for compliance
    for rec in result.recommendations {
        println!("Recommendation: {}", rec);
    }
}
```

---

*This is verifiable, safe, and extremely fast.*
*Last updated: December 28, 2025*


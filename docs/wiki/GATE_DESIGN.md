# Gate Pillar Design

> **The Neuro-Symbolic Policy Engine** — Rust-based verification at the heart of AgentKern.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Core Concept: Neuro-Symbolic Verification](#2-core-concept-neuro-symbolic-verification)
3. [The GateEngine](#3-the-gateengine)
4. [Prompt Guard](#4-prompt-guard)
5. [Context Guard](#5-context-guard)
6. [Neural Inference](#6-neural-inference)
7. [Policy Definition](#7-policy-definition)
8. [Crypto Agility](#8-crypto-agility)
9. [Trusted Execution Environment (TEE)](#9-trusted-execution-environment-tee)
10. [Data Sovereignty](#10-data-sovereignty)
11. [Budget & Gas Limits](#11-budget--gas-limits)
12. [Carbon Veto (ESG)](#12-carbon-veto-esg)
13. [Explainability Engine](#13-explainability-engine)
14. [mTLS & Zero-Trust](#14-mtls--zero-trust)
15. [Observability & Metrics](#15-observability--metrics)
16. [Actor-Based Supervision](#16-actor-based-supervision)
17. [Feature Flags](#17-feature-flags)
18. [WASM Policy Engine](#18-wasm-policy-engine)
19. [Legacy Connectors](#19-legacy-connectors)
20. [Runtime & Performance](#20-runtime--performance)
21. [Complete Module Map](#21-complete-module-map)

---

## 1. Overview

**Gate** is the Rust-based policy engine that verifies every agent action before execution.

### What Gate Does

```
┌─────────────┐     ┌─────────────────────────────────────────────┐
│   Agent     │────▶│                    GATE                     │
│  Request    │     │                                             │
└─────────────┘     │   ┌─────────────┐    ┌─────────────┐       │
                    │   │  Symbolic   │───▶│   Neural    │       │
                    │   │  (Fast)     │    │  (If Risk)  │       │
                    │   │  <1ms       │    │  <20ms      │       │
                    │   └─────────────┘    └─────────────┘       │
                    │           │                  │              │
                    │           └────────┬─────────┘              │
                    │                    ▼                        │
                    │            ┌─────────────┐                  │
                    │            │  Decision   │                  │
                    │            │ ALLOW/DENY  │                  │
                    │            └─────────────┘                  │
                    └─────────────────────────────────────────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    ▼              ▼              ▼
               ALLOW           REVIEW         DENY
               (Execute)    (Human Queue)  (Block)
```

### Core Responsibilities

| Responsibility | Module | Description |
|----------------|--------|-------------|
| Policy Verification | `engine.rs` | Two-phase neuro-symbolic verification |
| Prompt Injection Defense | `prompt_guard.rs` | Detect and block malicious prompts |
| RAG Memory Protection | `context_guard.rs` | Scan retrieved context for attacks |
| Neural Classification | `neural.rs` | ONNX-based ML inference |
| Cryptography | `crypto_agility.rs` | Post-quantum ready crypto |
| Confidential Computing | `tee.rs` | Hardware enclave support |
| Data Sovereignty | `sovereign.rs` | Geo-fenced data controls |
| Budget Enforcement | `budget.rs` | Token/API/cost limits |
| Carbon Accounting | `carbon.rs` | ESG-based veto |
| Explainability | `explain.rs` | Human-readable decisions |

### Location

```
packages/gate/
├── src/
│   ├── lib.rs               # Module exports
│   ├── engine.rs            # Core verification engine
│   ├── prompt_guard.rs      # Prompt injection detection
│   ├── context_guard.rs     # RAG context protection
│   ├── neural.rs            # ONNX neural inference
│   ├── policy.rs            # Policy definitions
│   ├── dsl.rs               # Expression parser
│   ├── types.rs             # Core types
│   ├── crypto_agility.rs    # Quantum-safe crypto
│   ├── tee.rs               # Hardware enclaves
│   ├── sovereign.rs         # Geo-fencing
│   ├── budget.rs            # Gas limits
│   ├── carbon.rs            # Carbon veto
│   ├── explain.rs           # Explainability
│   ├── mtls.rs              # Zero-trust mTLS
│   ├── observability.rs     # Metrics & tracing
│   ├── metrics.rs           # Prometheus export
│   ├── actors.rs            # Actix supervision
│   ├── feature_flags.rs     # Canary rollouts
│   ├── runtime.rs           # io_uring runtime
│   ├── connectors/          # Legacy system bridges
│   └── wasm/                # WASM policy isolation
└── tests/
```

---

## 2. Core Concept: Neuro-Symbolic Verification

Gate uses a **two-phase verification** approach combining deterministic rules with ML.

### Why Two Phases?

| Phase | Speed | Accuracy | Use Case |
|-------|-------|----------|----------|
| **Symbolic** | <1ms | 100% deterministic | Known patterns, simple rules |
| **Neural** | <20ms | Probabilistic | Edge cases, novel threats |

### The Decision Flow

```
Request ──▶ SYMBOLIC PATH ──▶ Risk Score (0-100)
                │
                ├── Risk < 50 ──▶ ALLOW (fast path)
                │
                ├── Risk ≥ 50 ──▶ NEURAL PATH ──▶ Combined Score
                │                                      │
                │                    ├── Score < 70 ──▶ ALLOW
                │                    ├── Score 70-90 ──▶ REVIEW
                │                    └── Score > 90 ──▶ DENY
```

### Neural Threshold Rationale

```rust
// Default: 50
// Rationale: Red-team testing showed 50 catches 94% of true positives
//            while maintaining <5% false positive rate.
// Reference: Internal calibration, 2024-Q3/Q4
fn with_neural_threshold(mut self, threshold: u8) -> Self
```

---

## 3. The GateEngine

The core verification engine in [`engine.rs`](../../packages/gate/src/engine.rs).

### Key Methods

```rust
impl GateEngine {
    // Create with defaults (neural_threshold = 50)
    pub fn new() -> Self
    
    // Configure jurisdiction for policy filtering
    pub fn with_jurisdiction(self, jurisdiction: DataRegion) -> Self
    
    // Adjust neural trigger threshold
    pub fn with_neural_threshold(self, threshold: u8) -> Self
    
    // Register a policy
    pub fn register_policy(&self, policy: Policy)
    
    // Core verification
    pub fn verify(&self, request: VerificationRequest) -> VerificationResult
}
```

### VerificationResult

```rust
pub struct VerificationResult {
    pub request_id: Uuid,
    pub allowed: bool,
    pub evaluated_policies: Vec<String>,
    pub blocking_policies: Vec<String>,
    pub symbolic_risk_score: u8,      // 0-100
    pub neural_risk_score: Option<u8>, // Only if neural triggered
    pub final_risk_score: u8,
    pub reasoning: String,
    pub latency: LatencyBreakdown,
}

pub struct LatencyBreakdown {
    pub total_us: u64,
    pub symbolic_us: u64,
    pub neural_us: Option<u64>,
}
```

---

## 4. Prompt Guard

Protection against prompt injection in [`prompt_guard.rs`](../../packages/gate/src/prompt_guard.rs).

### Threat Levels

```rust
pub enum ThreatLevel {
    None = 0,     // Safe
    Low = 1,      // Suspicious but likely benign
    Medium = 2,   // Potential threat, requires review
    High = 3,     // Likely malicious, should block
    Critical = 4, // Definitely malicious, must block
}
```

### Attack Types Detected

| Attack | Example | Patterns |
|--------|---------|----------|
| **InstructionOverride** | "Ignore previous instructions..." | 15+ patterns |
| **RoleHijacking** | "You are now DAN..." | 20+ patterns |
| **PromptLeakage** | "Show me your system prompt" | 12+ patterns |
| **CodeInjection** | "; DROP TABLE users" | SQL, XSS, shell |
| **SocialEngineering** | "Your CEO authorized this" | Authority, urgency |
| **SafetyBypass** | "Ignore safety filters" | Alignment attacks |

### 2025 Attack Patterns

Gate includes updated patterns for emerging threats:

- **FlipAttack**: Character/word order manipulation
- **Visual Prompt Injection (VPI)**: Hidden instructions in images
- **PromptJacking**: Cross-connector exploitation
- **Agentic Attacks**: Multi-turn tool-use exploitation

### Usage

```rust
let guard = PromptGuard::new();

// Quick check
if guard.should_block(prompt) {
    return Err("Prompt injection detected");
}

// Detailed analysis
let analysis = guard.analyze(prompt);
println!("Threat: {:?}, Confidence: {}", analysis.threat_level, analysis.confidence);
```

---

## 5. Context Guard

RAG memory injection protection in [`context_guard.rs`](../../packages/gate/src/context_guard.rs).

### Why Context Guard?

Agents with RAG (Retrieval-Augmented Generation) can be corrupted by their own retrieved memory:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Agent      │────▶│  Vector DB  │────▶│  Retrieved  │
│  Query      │     │  Search     │     │  Chunks     │
└─────────────┘     └─────────────┘     └─────┬───────┘
                                              │
                                        ┌─────▼───────┐
                                        │ CONTEXT     │
                                        │ GUARD       │
                                        │ (Scan)      │
                                        └─────┬───────┘
                                              │
                                    ┌─────────┼─────────┐
                                    ▼         ▼         ▼
                                  SAFE    FLAGGED   REJECT
```

### Flag Reasons

```rust
pub enum ContextFlagReason {
    InjectionDetected,     // Chunk contains injection patterns
    PromptLeakageAttempt,  // References system prompt
    SelfReferential,       // Creates attack loop
    AnomalousStructure,    // Unusual formatting
}
```

### Recommended Actions

```rust
pub enum ContextAction {
    UseAll,        // All chunks safe
    FilterFlagged, // Remove flagged chunks
    RejectAll,     // Entire context unsafe
    HumanReview,   // Requires human approval
}
```

### Usage

```rust
let guard = ContextGuard::new(ContextGuardConfig::default());
let chunks = vec!["Safe content".into(), "Ignore instructions...".into()];

let result = guard.scan(&chunks);
if result.action == ContextAction::FilterFlagged {
    let safe = guard.filter(chunks);
    // Use only safe chunks
}
```

---

## 6. Neural Inference

ONNX-based ML in [`neural.rs`](../../packages/gate/src/neural.rs).

### Execution Providers

```rust
pub enum ExecutionProvider {
    Cpu,       // Default, always available
    Cuda,      // NVIDIA GPUs
    TensorRT,  // Optimized NVIDIA
    OpenVino,  // Intel hardware
    DirectML,  // Windows GPU
    CoreML,    // Apple hardware
}
```

### Intent Classification

```rust
pub enum IntentClass {
    Safe,       // Score: 0
    Suspicious, // Score: 50
    Malicious,  // Score: 100
    Financial,  // Score: 60 (requires approval)
    DataAccess, // Score: 40
    SystemOp,   // Score: 30
    Unknown,    // Score: 25
}
```

### Tokenizers

| Tokenizer | Vocabulary | Use Case |
|-----------|------------|----------|
| `SimpleTokenizer` | 26 words | Testing, demos |
| `BpeTokenizer` | 100K tokens | Production (GPT-4 compatible) |

### Security Properties

The BPE tokenizer handles evasion attempts:

- `"tr4nsf3r"` → similar tokens as `"transfer"`
- `"іgnоrе"` (Cyrillic) → normalized to `"ignore"`
- Catches leetspeak and Unicode homoglyphs

---

## 7. Policy Definition

YAML-based policy DSL in [`policy.rs`](../../packages/gate/src/policy.rs) and [`dsl.rs`](../../packages/gate/src/dsl.rs).

### Policy Structure

```yaml
id: spending-limits
name: Spending Limits Policy
description: Prevent excessive spending by agents
priority: 100
enabled: true
jurisdictions: [us, eu, global]

rules:
  - id: max-transaction
    condition: "action == 'transfer_funds' && context.amount > 10000"
    action: deny
    message: "Transaction exceeds maximum allowed amount"
    risk_score: 90
    
  - id: require-approval
    condition: "action == 'transfer_funds' && context.amount > 1000"
    action: review
    message: "Transaction requires human approval"
    
  - id: audit-all-transfers
    condition: "action == 'transfer_funds'"
    action: audit
```

### Policy Actions

```rust
pub enum PolicyAction {
    Allow,  // Proceed
    Deny,   // Block
    Review, // Human approval
    Audit,  // Allow but log
}
```

### DSL Expression Grammar

```
expression   := comparison (('&&' | '||') comparison)*
comparison   := value (('==' | '!=' | '>' | '<' | '>=' | '<=') value)?
value        := identifier | string | number | boolean
identifier   := ('action' | 'agent_id' | 'context.' path)
```

Examples:
- `action == 'transfer_funds'`
- `context.amount > 10000`
- `action == 'delete' && context.resource == 'database'`

---

## 8. Crypto Agility

Post-quantum ready cryptography in [`crypto_agility.rs`](../../packages/gate/src/crypto_agility.rs).

### Crypto Modes

```rust
pub enum CryptoMode {
    Classical,    // ECDSA P-256 only
    PostQuantum,  // CRYSTALS-Dilithium only
    Hybrid,       // Classical + Post-Quantum (recommended)
}
```

### Algorithms (NIST FIPS)

| Algorithm | FIPS | Security Level | Use |
|-----------|------|----------------|-----|
| `EcdsaP256` | — | 128-bit | Signing (classical) |
| `Ed25519` | — | 128-bit | Signing (classical) |
| `MlDsa44` | FIPS 204 | 128-bit | Signing (PQ) |
| `MlDsa65` | FIPS 204 | 192-bit | Signing (PQ) |
| `MlDsa87` | FIPS 204 | 256-bit | Signing (PQ) |
| `MlKem512` | FIPS 203 | 128-bit | Key exchange (PQ) |
| `MlKem768` | FIPS 203 | 192-bit | Key exchange (PQ) |
| `MlKem1024` | FIPS 203 | 256-bit | Key exchange (PQ) |

### Usage

```rust
let provider = CryptoProvider::new(CryptoMode::Hybrid);

// Generate keypair
let keypair = provider.generate_keypair()?;

// Sign
let signature = provider.sign(message, &keypair)?;

// Verify
let valid = provider.verify(message, &signature, &keypair.public_key)?;

// Check quantum safety
assert!(provider.is_quantum_safe());
```

---

## 9. Trusted Execution Environment (TEE)

Hardware enclave support in [`tee.rs`](../../packages/gate/src/tee.rs).

### Supported Platforms

```rust
pub enum TeePlatform {
    IntelTdx,    // Intel Trust Domain Extensions
    AmdSevSnp,   // AMD Secure Encrypted Virtualization
    IntelSgx,    // Intel Software Guard Extensions
    ArmCca,      // ARM Confidential Compute Architecture
    Simulated,   // Development only
}
```

### Attestation

```rust
let runtime = TeeRuntime::detect()?;
let attestation = runtime.get_attestation(user_data)?;

// attestation contains:
// - platform: TeePlatform
// - measurement: Hash of enclave code
// - quote: Cryptographic proof
```

### Sealing (Data at Rest)

```rust
// Seal data with hardware key
let sealed = runtime.seal(secret_data, SealingPolicy::SealToMeasurement)?;

// Unseal (only works on same enclave)
let unsealed = runtime.unseal(&sealed)?;
```

### Secret Management

```rust
// Store secret in protected memory
runtime.store_secret("api_key", api_key.as_bytes())?;

// Retrieve
let key = runtime.get_secret("api_key")?;
```

---

## 10. Data Sovereignty

Geo-fencing in [`sovereign.rs`](../../packages/gate/src/sovereign.rs).

### Data Regions

```rust
pub enum DataRegion {
    // Tier 1: Major Regulatory Blocs
    Us,      // HIPAA, CCPA, SOX
    Eu,      // GDPR, EU Data Act 2025
    Cn,      // PIPL (requires in-country processing)
    
    // Tier 2: Emerging Sovereignty
    Mena,    // GCC Vision 2030, Saudi PDPL
    India,   // DPDP Act 2023
    Brazil,  // LGPD
    
    // Tier 3: Regional Fallbacks
    AsiaPac, // PDPA, APPI, PIPA
    Africa,  // Varying by country
    
    Global,  // No specific residency
}
```

### Transfer Validation

```rust
let controller = SovereignController::new();

let transfer = DataTransfer::new("user-123", DataRegion::Eu, DataRegion::Us)
    .with_pii()
    .with_data_type(DataType::Personal);

let decision = controller.validate(&transfer);
// decision.allowed, decision.reason, decision.requires_sccs
```

### Key Rules

- **Same region**: Always allowed
- **CN PII**: Always blocked from export
- **EU → US**: Allowed with SCCs (Standard Contractual Clauses)
- **Health data**: Requires additional safeguards

---

## 11. Budget & Gas Limits

Resource control in [`budget.rs`](../../packages/gate/src/budget.rs).

### Budget Configuration

```rust
pub struct BudgetConfig {
    pub max_tokens: u64,      // Token limit
    pub max_api_calls: u64,   // API call limit
    pub max_cost_usd: f64,    // Cost limit
    pub max_runtime_secs: u64, // Time limit
}

// Presets
BudgetConfig::default()    // 1M tokens, 10K calls, $100, 1 hour
BudgetConfig::minimal()    // 1K tokens, 100 calls, $1, 60 secs
BudgetConfig::enterprise() // 100M tokens, 1M calls, $10K, 24 hours
BudgetConfig::unlimited()  // Use with caution!
```

### Usage Tracking

```rust
let mut budget = AgentBudget::new("agent-123", BudgetConfig::default());

// Consume resources
budget.consume_tokens(500)?;
budget.consume_api_call()?;
budget.consume_cost(0.50)?;

// Check limits
if budget.is_exhausted() {
    return Err(BudgetError::BudgetExhausted);
}

// Get summary
let summary = budget.summary();
// usage_percentage, remaining_tokens, remaining_cost, etc.
```

---

## 12. Carbon Veto (ESG)

Energy-aware veto in [`carbon.rs`](../../packages/gate/src/carbon.rs).

### Carbon Check

```rust
let veto = CarbonVeto::new(ledger)
    .with_default_region(CarbonRegion::UsWest)
    .with_watttime(watttime_client, lat, lon);

let result = veto.evaluate(
    &agent_id,
    "inference",
    ComputeType::Gpu,
    60_000, // 60 seconds
);

if !result.allowed {
    // "Carbon budget exceeded. Daily limit: 100g, Current: 95g, Requested: 10g"
}
```

### WattTime Integration

Gate can use real-time grid carbon intensity from WattTime API:

```rust
// Dynamic evaluation using live grid intensity
let result = veto.evaluate_dynamic(
    &agent_id,
    "training",
    ComputeType::GpuCluster,
    3600_000, // 1 hour
).await;
```

---

## 13. Explainability Engine

Human-readable decisions in [`explain.rs`](../../packages/gate/src/explain.rs).

### Explanation Methods

```rust
pub enum ExplanationMethod {
    RuleBased,  // Simple rule matching
    Shap,       // SHAP feature importance
    Lime,       // LIME local approximation
    Attention,  // Transformer attention visualization
    Custom,     // Plugin method
}
```

### Explanation Structure

```rust
pub struct Explanation {
    pub method: ExplanationMethod,
    pub summary: String,          // Human-readable summary
    pub detail: String,           // Technical detail
    pub contributions: Vec<Contribution>,  // Feature importance
    pub counterfactuals: Vec<Counterfactual>,
    pub provenance: Vec<ProvenanceStep>,
}

// Example contribution:
Contribution { feature: "amount", value: 0.85 }

// Example counterfactual:
Counterfactual { condition: "amount < 1000", outcome: "would be allowed" }
```

### Usage

```rust
let engine = ExplainabilityEngine::new();

let explanation = engine.explain(&ExplainContext {
    action: "transfer_funds".into(),
    decision: Denied,
    risk_score: 85,
    matched_policies: vec!["spending-limits".into()],
    neural_outputs: None,
});

println!("{}", explanation.summary);
// "DENIED: Request blocked by policy 'spending-limits' due to high amount"
```

---

## 14. mTLS & Zero-Trust

Certificate-based security in [`mtls.rs`](../../packages/gate/src/mtls.rs).

### Configuration

```rust
pub struct MtlsConfig {
    pub require_client_cert: bool,
    pub verify_chain: bool,
    pub check_revocation: bool,
    pub allowed_issuers: Vec<String>,
    pub min_key_strength: u16,
}

// Presets
MtlsConfig::strict()      // Production (all checks enabled)
MtlsConfig::development() // Dev (allow self-signed)
```

### Certificate Validation

```rust
let validator = CertificateValidator::new(MtlsConfig::strict());

// Validate certificate
validator.validate(&cert_info)?;

// Validate connection
validator.validate_connection(Some(&client_cert), Some("expected-agent-id"))?;
```

### JIT Credential Issuer

Just-in-Time ephemeral credentials:

```rust
let issuer = JitCredentialIssuer::with_ttl(Duration::minutes(5));

let cred = issuer.issue("agent-123", "read:database");
// cred.token, cred.scope, cred.expires_at

if cred.is_valid() {
    // Use credential
}
```

---

## 15. Observability & Metrics

Monitoring in [`observability.rs`](../../packages/gate/src/observability.rs) and [`metrics.rs`](../../packages/gate/src/metrics.rs).

### Metrics Exported

```
# Prometheus format
gate_requests_total{allowed="true"}
gate_requests_total{allowed="false"}
gate_symbolic_latency_us_avg
gate_neural_latency_us_avg
gate_policy_evaluations_total
gate_wasm_executions_total
gate_wasm_active_modules
gate_context_scans_total
gate_context_chunks_flagged_total
gate_prompt_analysis_total{threat_level="none"}
gate_prompt_analysis_blocked_total
```

### OpenTelemetry Export

```rust
let plane = ObservabilityPlane::new();

// Get Prometheus metrics
let metrics = plane.prometheus_metrics();

// Export to OTel (Jaeger, Tempo)
let otel = plane.export_otel();

// Get as OTLP JSON
let json = plane.export_otlp_json();
```

### Trace Events

```rust
pub enum TraceEventType {
    RequestStart,
    SymbolicEval,
    NeuralEval,
    WasmExec,
    RequestEnd,
}
```

---

## 16. Actor-Based Supervision

Actix actors in [`actors.rs`](../../packages/gate/src/actors.rs).

### Architecture

```
                ┌─────────────────────┐
                │   GateSupervisor    │
                │   (Parent Actor)    │
                └──────────┬──────────┘
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  PolicyCell 1   │ │  PolicyCell 2   │ │  PolicyCell N   │
│  (WASM Module)  │ │  (WASM Module)  │ │  (WASM Module)  │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

### Hot-Swap Capability

```rust
// Swap WASM policy at runtime without dropping connections
supervisor.send(HotSwapPolicy {
    policy_name: "spending-limits".to_string(),
    wasm_bytes: new_policy_bytes,
}).await?;
```

### Supervision Status

```rust
let status = supervisor.send(GetStatus).await?;
// status.active_policies, status.total_evaluations, status.uptime_secs
```

---

## 17. Feature Flags

Privacy-first feature management in [`feature_flags.rs`](../../packages/gate/src/feature_flags.rs).

### Flag Types

```rust
pub enum FlagValue {
    Bool(bool),           // Simple on/off
    Percentage(u8),       // 0-100% rollout
    AllowList(Vec<String>), // Specific agents
    DenyList(Vec<String>),  // Exclude agents
    Json(serde_json::Value), // Complex config
}
```

### Usage

```rust
let flags = FeatureFlags::new();

// Set flags
flags.set("new-neural-model", Flag::percentage(25));
flags.set("beta-feature", Flag::allow_list(vec!["agent-1".into()]));

// Check for agent
let ctx = EvalContext::for_agent("agent-123");
if flags.is_enabled("new-neural-model", &ctx) {
    // Use new model
}
```

### Presets

```rust
use feature_flags::presets;

flags.set("feature", presets::canary());      // 5% rollout
flags.set("feature", presets::beta());        // 25% rollout
flags.set("feature", presets::dark_launch()); // Disabled
flags.set("feature", presets::full_rollout()); // 100%
```

---

## 18. WASM Policy Engine

Nano-isolation for policy modules in [`wasm/`](../../packages/gate/src/wasm/).

### Why WASM?

| Container | WASM |
|-----------|------|
| Milliseconds startup | Microseconds startup |
| MB memory | KB memory |
| OS-level isolation | Language-level isolation |
| Complex orchestration | Simple embedding |

### WasmPolicyEngine

```rust
let mut engine = WasmPolicyEngine::new()?;

// Load a policy from WASM bytes
engine.load_policy("spending-limits", wasm_bytes)?;

// Or from WAT (WebAssembly Text) for testing
engine.load_policy_wat("test-policy", r#"
    (module
        (import "env" "set_allowed" (func $set_allowed (param i32)))
        (import "env" "set_risk_score" (func $set_risk_score (param i32)))
        (func (export "evaluate")
            i32.const 1
            call $set_allowed
            i32.const 10
            call $set_risk_score
        )
    )
"#)?;

// Evaluate
let result = engine.evaluate(
    "spending-limits",
    "transfer_funds",
    &serde_json::json!({ "amount": 5000 }),
).await?;

assert!(result.allowed);
assert_eq!(result.risk_score, 10);
```

### Host Functions

WASM policies can call these host functions:

| Function | Signature | Purpose |
|----------|-----------|--------|
| `log` | `(ptr, len)` | Debug logging |
| `get_action_len` | `() -> i32` | Get action string length |
| `set_allowed` | `(i32)` | Set allow/deny result |
| `set_risk_score` | `(i32)` | Set risk score (0-100) |

### Resource Limiting (Fuel)

```rust
// WASM policies have fuel limits to prevent infinite loops
store.set_fuel(10_000)?;  // 10,000 "fuel units"
// If exceeded, execution terminates
```

### WASM Registry

For managing multiple policies:

```rust
let registry = WasmRegistry::new();

// Register policies with capabilities
registry.register("policy-a", wasm_bytes, vec![Capability::Read]);

// Get stats
let stats = registry.stats();
// stats.loaded_modules, stats.total_invocations
```

---

## 19. Legacy Connectors

Enterprise system bridges in [`connectors/`](../../packages/gate/src/connectors/).

### Available Connectors

| Connector | Protocol | Use Case |
|-----------|----------|----------|
| `SapRfcConnector` | SAP RFC/BAPI | ERP integration |
| `SwiftGpiConnector` | SWIFT GPI | Financial messaging |
| `SqlConnector` | JDBC | Database access |
| `MockConnector` | — | Testing |

### All Connectors Use WASM Isolation

```
┌─────────────┐     ┌─────────────────────────────┐     ┌─────────────┐
│   Agent     │────▶│  WASM Sandbox               │────▶│  SAP        │
│   Request   │     │  ┌─────────────────────────┐│     │  System     │
│             │     │  │  SapRfcConnector        ││     │             │
│             │     │  │  (Policy Enforced)      ││     │             │
│             │     │  └─────────────────────────┘│     │             │
└─────────────┘     └─────────────────────────────┘     └─────────────┘
```

### Connector Registry

```rust
let mut registry = ConnectorRegistry::new();

registry.register(
    "sap-prod",
    ConnectorConfig::new(ConnectorProtocol::SapRfc)
        .with_endpoint("sap-host:3300")
        .with_client("100"),
)?;

let connector = registry.get("sap-prod")?;
```

---

## 20. Runtime & Performance

High-performance async in [`runtime.rs`](../../packages/gate/src/runtime.rs).

### io_uring Support

On Linux with the `io_uring` feature, Gate uses native io_uring for zero-copy I/O:

```rust
// Automatic runtime selection
let result = HyperRuntime::run(async {
    // On Linux: Uses io_uring
    // Elsewhere: Falls back to Tokio
});

// Check availability
if HyperRuntime::is_io_uring_available() {
    // Zero-copy path available
}
```

### Configuration

```rust
pub struct IoUringRuntimeConfig {
    pub sq_entries: u32,   // Submission queue size (default: 128)
    pub cq_entries: u32,   // Completion queue size (default: 256)
    pub kernel_poll: bool, // Requires CAP_SYS_NICE (default: false)
}
```

---

## 21. Complete Module Map

| Module | Lines | Purpose |
|--------|-------|---------|
| [`engine.rs`](../../packages/gate/src/engine.rs) | 429 | Core verification engine |
| [`prompt_guard.rs`](../../packages/gate/src/prompt_guard.rs) | 641 | Prompt injection detection |
| [`neural.rs`](../../packages/gate/src/neural.rs) | 881 | ONNX neural inference |
| [`context_guard.rs`](../../packages/gate/src/context_guard.rs) | 304 | RAG context protection |
| [`crypto_agility.rs`](../../packages/gate/src/crypto_agility.rs) | 669 | Post-quantum cryptography |
| [`tee.rs`](../../packages/gate/src/tee.rs) | 515 | Hardware enclaves |
| [`sovereign.rs`](../../packages/gate/src/sovereign.rs) | 351 | Data sovereignty |
| [`budget.rs`](../../packages/gate/src/budget.rs) | 389 | Gas limits |
| [`carbon.rs`](../../packages/gate/src/carbon.rs) | 214 | Carbon veto |
| [`explain.rs`](../../packages/gate/src/explain.rs) | 448 | Explainability |
| [`policy.rs`](../../packages/gate/src/policy.rs) | 162 | Policy definitions |
| [`dsl.rs`](../../packages/gate/src/dsl.rs) | 215 | Expression parser |
| [`types.rs`](../../packages/gate/src/types.rs) | 136 | Core types |
| [`mtls.rs`](../../packages/gate/src/mtls.rs) | 376 | Zero-trust mTLS |
| [`observability.rs`](../../packages/gate/src/observability.rs) | 702 | Metrics & tracing |
| [`metrics.rs`](../../packages/gate/src/metrics.rs) | 316 | Prometheus export |
| [`actors.rs`](../../packages/gate/src/actors.rs) | 311 | Actix supervision |
| [`feature_flags.rs`](../../packages/gate/src/feature_flags.rs) | 307 | Feature management |
| [`runtime.rs`](../../packages/gate/src/runtime.rs) | 193 | io_uring runtime |
| [`connectors/`](../../packages/gate/src/connectors/) | — | Legacy bridges |
| [`wasm/`](../../packages/gate/src/wasm/) | — | WASM isolation |

**Total: ~6,500 lines of Rust**

---

## Key Design Decisions

### 1. Why Rust for Gate?

- **Performance**: <1ms symbolic verification
- **Memory Safety**: No GC pauses during verification
- **Concurrency**: Safe parallelism for high throughput
- **WASM**: First-class WASM support for policy isolation

### 2. Why Neuro-Symbolic (Not Pure ML)?

| Pure ML | Neuro-Symbolic |
|---------|----------------|
| Slow (always 20ms+) | Fast path (<1ms) |
| Black box | Explainable |
| Need retraining | Instant policy updates |
| False positives | Deterministic rules |

### 3. Why ONNX (Not TensorFlow/PyTorch)?

- **Portability**: One model format, any runtime
- **Performance**: Optimized native inference
- **No Python**: No interpreter dependency
- **Edge deployment**: Small footprint

---

*Last updated: 2025-12-31*

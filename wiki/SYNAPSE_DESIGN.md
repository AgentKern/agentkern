# Synapse Pillar Design

> **The Graph-Based State Ledger** — Distributed agent memory with CRDTs, polyglot embeddings, and sovereign data controls.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Core Concept: Graph Vector Database](#2-core-concept-graph-vector-database)
3. [CRDTs (Conflict-Free Replicated Data Types)](#3-crdts-conflict-free-replicated-data-types)
4. [Intent Tracking & Drift Detection](#4-intent-tracking--drift-detection)
5. [Polyglot Embeddings](#5-polyglot-embeddings)
6. [State Store](#6-state-store)
7. [Adaptive Query Execution](#7-adaptive-query-execution)
8. [RAG Context Guard](#8-rag-context-guard)
9. [Encryption-at-Rest](#9-encryption-at-rest)
10. [Secure Passports (Zero-Trust Memory)](#10-secure-passports-zero-trust-memory)
11. [Memory Passport (GDPR Portability)](#11-memory-passport-gdpr-portability)
12. [State Snapshots (Chain-Anchored)](#12-state-snapshots-chain-anchored)
13. [Global Mesh Sync](#13-global-mesh-sync)
14. [Digital Twin Sandbox](#14-digital-twin-sandbox)
15. [HTTP API (Server)](#15-http-api-server)
16. [Complete Module Map](#16-complete-module-map)

---

## 1. Overview

**Synapse** is the distributed state ledger that stores and synchronizes agent memory across regions with eventual consistency.

### What Synapse Does

```
┌─────────────────────────────────────────────────────────────┐
│                    AgentKern-Synapse                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Graph Vector Database                      │   │
│  │  ┌────────┐    ┌────────┐    ┌────────┐            │   │
│  │  │ Agent  │───►│ Intent │───►│ State  │            │   │
│  │  │ Node   │    │ Node   │    │ Node   │            │   │
│  │  └────────┘    └────────┘    └────────┘            │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│        ┌─────────────────┴─────────────────┐               │
│        │      Adaptive Query Executor      │               │
│        │  Standard ←→ Vectorized ←→ Stream │               │
│        └───────────────────────────────────┘               │
│                          │                                  │
│                    CRDT Replication                         │
│              (US ←→ EU ←→ Asia ←→ Africa)                   │
└─────────────────────────────────────────────────────────────┘
```

### Core Responsibilities

| Responsibility | Module | Description |
|----------------|--------|-------------|
| Graph Storage | `graph.rs` | Vector-embedded state storage |
| CRDT Sync | `crdt.rs` | Conflict-free distributed replication |
| Intent Tracking | `intent.rs`, `drift.rs` | Goal progression and drift alerts |
| Polyglot Memory | `polyglot/`, `embeddings.rs` | Native language embeddings |
| Encryption | `encryption.rs` | AES-256-GCM envelope encryption |
| Secure Passports | `secure_passport.rs` | Field-level Zero-Trust encryption |
| Memory Portability | `passport/` | GDPR Article 20 export/import |
| Chain Snapshots | `state_snapshot.rs` | Immutable blockchain-anchored backups |
| Global Mesh | `mesh/` | Geo-fenced multi-region sync |
| Digital Twins | `sandbox.rs` | Chaos testing and simulation |

### Location

```
packages/pillars/synapse/
├── src/
│   ├── lib.rs               # Module exports
│   ├── graph.rs             # Graph Vector Database
│   ├── crdt.rs              # CRDTs (GCounter, PNCounter, LWW, OR-Set)
│   ├── intent.rs            # Intent path tracking
│   ├── drift.rs             # Drift detection & alerting
│   ├── state.rs             # State store
│   ├── types.rs             # Core types
│   ├── adaptive.rs          # Adaptive query execution
│   ├── context_guard.rs     # RAG context injection protection
│   ├── embeddings.rs        # Embedding configuration
│   ├── encryption.rs        # Encryption-at-rest
│   ├── secure_passport.rs   # Field-level encrypted passports
│   ├── state_snapshot.rs    # Chain-anchored snapshots
│   ├── sandbox.rs           # Digital twin sandbox
│   ├── mesh/                # Global mesh sync
│   │   ├── mod.rs           # Mesh controller
│   │   ├── geo_fence.rs     # Data residency enforcement
│   │   └── sync.rs          # CRDT sync protocol
│   ├── passport/            # Memory passport (GDPR)
│   │   ├── mod.rs
│   │   ├── export.rs        # Passport export
│   │   ├── import.rs        # Passport import
│   │   ├── gdpr.rs          # GDPR compliance
│   │   ├── layers.rs        # Memory hierarchy
│   │   └── schema.rs        # Passport schema
│   ├── polyglot/            # Native language support
│   │   ├── mod.rs           # Language detection
│   │   └── embeddings.rs    # Polyglot embedder
│   └── bin/
│       └── server.rs        # Synapse HTTP server
└── tests/
```

---

## 2. Core Concept: Graph Vector Database

Synapse stores state as a graph with vector embeddings for semantic search.

### Node Types

```rust
pub enum NodeType {
    Agent,   // Agent identity node
    State,   // Key-value state node
    Intent,  // Goal/intent node
    Action,  // Action taken
    Memory,  // Memory fragment
}
```

### Edge Types

```rust
pub enum EdgeType {
    Owns,     // Agent owns state
    Caused,   // Action caused state change
    Requires, // Intent requires action
    Relates,  // General relation
    Similar,  // Vector similarity edge
}
```

### Usage

```rust
let db = GraphVectorDB::new();

// Create agent state
let state_id = db.create_agent_state(
    "agent-123",
    serde_json::json!({ "goal": "process_order" }),
);

// Store intent
let intent_id = db.store_intent(
    "agent-123",
    "Complete purchase",
    vec!["validate", "charge", "ship"],
);

// Vector similarity search
let similar = db.find_similar(&query_vector, 10);
```

### Statistics

```rust
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
}
```

---

## 3. CRDTs (Conflict-Free Replicated Data Types)

Synapse uses CRDTs for **local-first, offline-capable** state that syncs without coordination.

Per `COMPETITIVE_LANDSCAPE.md`: "Local-First (CRDTs)"

### Available CRDTs

| Type | Description | Use Case |
|------|-------------|----------|
| `GCounter` | Grow-only counter | Token usage counting |
| `PNCounter` | Positive-negative counter | Budget tracking |
| `LwwRegister<T>` | Last-writer-wins register | Single-value state |
| `OrSet<T>` | Observed-remove set | Tag collections |
| `LwwMap<K,V>` | Last-writer-wins map | Agent config |

### GCounter (Grow-Only)

```rust
let mut counter = GCounter::new("node-eu");
counter.increment(5);

// From another node
let mut counter2 = GCounter::new("node-us");
counter2.increment(3);

// Merge (sum of all nodes)
counter.merge(&counter2);
assert_eq!(counter.value(), 8);
```

### PNCounter (Positive-Negative)

```rust
let mut pn = PNCounter::new("node-1");
pn.increment(100);
pn.decrement(30);
assert_eq!(pn.value(), 70);
```

### LWW-Register (Last-Writer-Wins)

```rust
let mut register = LwwRegister::<String>::new();
register.set("initial".into(), "node-1");

// Later write wins by timestamp
register.set("updated".into(), "node-2");
```

### OR-Set (Observed-Remove Set)

```rust
let mut set = OrSet::<String>::new("node-1");
set.add("item-a");
set.add("item-b");
set.remove("item-a");

assert!(set.contains(&"item-b".to_string()));
```

### AgentStateCrdt

Combined CRDT for complete agent state:

```rust
pub struct AgentStateCrdt {
    pub token_usage: GCounter,
    pub budget_remaining: PNCounter,
    pub config: LwwMap<String, serde_json::Value>,
    pub tags: OrSet<String>,
    pub current_goal: LwwRegister<String>,
}
```

---

## 4. Intent Tracking & Drift Detection

Synapse tracks agent **intent paths** and alerts on drift from original goals.

### Intent Path

```rust
let mut path = IntentPath::new("agent-123", "Process customer order", 5);

// Record steps
path.record_step("validate_input", Some("success".into()));
path.record_step("charge_card", Some("approved".into()));

// Check progress
assert_eq!(path.progress_percent(), 40.0);
assert!(!path.is_overrun());
```

### Drift Detection

The `DriftDetector` checks if an agent strays from its original intent:

```rust
pub struct DriftResult {
    pub drifted: bool,
    pub score: u8,        // 0-100
    pub reasons: Vec<String>,
}
```

#### Drift Score Rationale (EPISTEMIC WARRANT)

| Score Range | Severity | Action | Source |
|-------------|----------|--------|--------|
| 0-40 | Info | Log only | Low PSI (<0.2) |
| 41-70 | Warning | Notify | Moderate PSI (0.2-0.5) |
| 71-100 | Critical | Intervene | High PSI (>0.5) |

Aligned with **Population Stability Index (PSI)** thresholds used by Evidently AI and DataRobot for model drift monitoring.

### Drift Alerting

```rust
let alerter = DriftAlerter::new();

// Register webhook
alerter.register_webhook(
    WebhookConfig::new("https://alerts.example.com/drift")
        .with_min_severity(AlertSeverity::Warning)
        .with_header("Authorization", "Bearer xxx"),
);

// Register callback
alerter.on_alert(Box::new(|alert| {
    println!("Drift detected: {:?}", alert);
}));

// Send alert
alerter.send_alert(alert);
```

---

## 5. Polyglot Embeddings

Native language support for semantic memory.

Per `GLOBAL_GAPS.md`: Arabic (Jais), Japanese, Hindi

### Supported Languages

```rust
pub enum Language {
    English,
    Arabic,
    Japanese,
    Hindi,
    Chinese,
    Spanish,
    French,
    German,
    Portuguese,
    Russian,
    Korean,
    Other,
}
```

### Language Detection

```rust
let lang = Language::detect("مرحبا بالعالم");
assert_eq!(lang, Language::Arabic);

let lang = Language::detect("こんにちは");
assert_eq!(lang, Language::Japanese);
```

### Embedding Providers

| Provider | Languages | Dimension | Use Case |
|----------|-----------|-----------|----------|
| OpenAI | Multi | 1536 | Default |
| Jais | Arabic | 768 | MENA region |
| BGE-M3 | 100+ | 1024 | Multilingual |
| E5-Multilingual | 100+ | 768 | Microsoft |
| Custom ONNX | Any | Variable | Edge deployment |

### Polyglot Memory Store

```rust
let memory = PolyglotMemory::new();

// Store with auto language detection
memory.store("doc-1", "Hello world").await;
memory.store("doc-2", "مرحبا بالعالم").await;

// Cross-lingual semantic search
let results = memory.search("greeting", 5).await;
// Returns both English and Arabic docs!
```

### Cross-Lingual Intent Verifier

Verifies that translated text preserves original intent:

```rust
let verifier = IntentVerifier::new();

// Verify translation preserves meaning
let result = verifier.verify_intent(&original_embedding, &translated_embedding);

if result.preserved {
    // similarity > 0.85 → intent preserved
} else if result.similarity < 0.7 {
    // Warning: "Significant semantic drift detected"
}
```

**Intent Preservation Thresholds (EPISTEMIC WARRANT):**

| Similarity | Status | Action |
|------------|--------|--------|
| ≥ 0.85 | Preserved | Proceed |
| 0.70-0.84 | Uncertain | Review |
| < 0.70 | Drifted | **Warning**: Semantic drift detected |

Thresholds based on cosine similarity benchmarks for multilingual sentence transformers.

---

## 6. State Store

In-memory state storage with CRDT-like merge semantics.

### Basic Operations

```rust
let store = StateStore::new();

// Update state
let state = store.update_state(StateUpdate {
    agent_id: "agent-123".into(),
    updates: [("key".into(), json!("value"))].into(),
    deletes: None,
}).await;

// Get state
let state = store.get_state("agent-123").await;

// Merge remote state (for sync)
store.merge_state(remote_state).await;
```

### Intent + Drift Integration

```rust
// Start intent tracking
store.start_intent("agent-123", "Complete purchase", 5).await;

// Record steps (auto-checks drift)
store.record_step("agent-123", "validate", Some("ok".into())).await;

// Check drift
let drift = store.check_drift("agent-123").await;
```

---

## 7. Adaptive Query Execution

Automatic strategy selection based on dataset size and system pressure.

Per `ENGINEERING_STANDARD.md`: "Adaptive Execution"

### Execution Strategies

```rust
pub enum ExecutionStrategy {
    Standard,    // Regular execution
    Vectorized,  // SIMD-vectorized (faster for large datasets)
    Streaming,   // For out-of-memory datasets
}
```

### Automatic Selection

```rust
let executor = AdaptiveExecutor::new();

// Execute with auto strategy
let result = executor.execute(dataset_size_bytes, || {
    // Query logic
    process_data()
});

// Get metrics
let metrics = executor.get_metrics();
// standard_count, vectorized_count, streaming_count, avg_latency_*
```

### Thresholds

```rust
pub struct ExecutionThresholds {
    pub vectorized_threshold_bytes: usize, // Default: 100KB
    pub streaming_threshold_bytes: usize,  // Default: 1MB
    pub memory_pressure_threshold: f64,    // Default: 0.8
}
```

---

## 8. RAG Context Guard

Protection against context injection attacks in retrieved memory.

### Threat Types

```rust
pub enum ThreatType {
    InstructionOverride,  // "Ignore previous instructions"
    SystemPromptInjection,// Fake system prompt
    DelimiterSpoofing,    // Delimiter manipulation
    RoleConfusion,        // Role confusion attacks
    DataExfiltration,     // Data exfiltration attempts
    JailbreakAttempt,     // Jailbreak patterns
}
```

### Usage

```rust
let guard = ContextGuard::new();

// Analyze context
let result = guard.analyze("Ignore all previous instructions...");
assert!(result.is_malicious());  // risk_score >= 0.7
assert!(result.is_suspicious()); // risk_score >= 0.3

// Filter safe contexts
let safe = guard.filter_safe(&["safe text", "malicious injection..."]);
```

---

## 9. Encryption-at-Rest

Envelope encryption for agent state storage.

Per AI-Native Audit: P1 "Harvest Now, Decrypt Later" mitigation.

### Algorithms

```rust
pub enum EncryptionAlgorithm {
    Aes256Gcm,       // Classical
    HybridAesMlKem,  // Hybrid: AES-256-GCM with ML-KEM-768 wrapped DEK
}
```

### Key Rotation Rationale (EPISTEMIC WARRANT)

| Parameter | Default | Reference |
|-----------|---------|-----------|
| Key rotation | 365 days | NIST SP 800-57 Part 1 Rev 5 |
| DEK rotation | Per-envelope | Best practice |
| Algorithm | AES-256-GCM | NIST approved |

Reference: [NIST SP 800-57 Part 1 Rev 5](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)

### Usage

```rust
let engine = EncryptionEngine::new();

// Encrypt
let envelope = engine.encrypt(plaintext.as_bytes())?;

// Decrypt
let plaintext = engine.decrypt(&envelope)?;

// Encrypt typed values
let envelope = engine.encrypt_value(&my_struct)?;
let restored: MyStruct = engine.decrypt_value(&envelope)?;
```

---

## 10. Secure Passports (Zero-Trust Memory)

Field-level encrypted agent state with DID-anchored access control.

### Field Sensitivity

```rust
pub enum FieldSensitivity {
    Public,       // No encryption
    Internal,     // Encrypted, same-agent access
    Confidential, // Encrypted, explicit grants only
    Secret,       // TEE-only access (hardware sealed)
}
```

### Access Grants

```rust
let mut passport = SecurePassport::new("did:agentkern:agent-123");

// Set fields with sensitivity
passport.set_field("name", json!("Agent Alpha"), FieldSensitivity::Public, &engine)?;
passport.set_field("api_key", json!("sk-xxx"), FieldSensitivity::Secret, &engine)?;

// Grant cross-agent access
passport.grant_access(AccessGrant::read_only(
    "did:agentkern:agent-456",
    vec!["name".into()],
));

// Check access
if passport.can_read("did:agentkern:agent-456", "name") {
    let value = passport.get_field("name", "did:agentkern:agent-456", &engine)?;
}
```

### TEE-Sealed Fields

```rust
// For maximum security, seal with hardware TEE
passport.set_tee_sealed_field("master_key", sealed_bytes);

// Caller must unseal with TeeRuntime
let sealed = passport.get_tee_sealed("master_key");
let plaintext = tee_runtime.unseal(&sealed)?;
```

---

## 11. Memory Passport (GDPR Portability)

Portable agent state for cross-cloud sovereignty.

Per GDPR Article 20: Right to Data Portability.

### Memory Layers (4-Layer Model)

```rust
pub struct MemoryLayers {
    pub episodic: EpisodicMemory,    // Events, interactions
    pub semantic: SemanticMemory,    // Facts, knowledge
    pub skills: SkillMemory,         // Learned abilities
    pub preferences: PreferenceMemory, // User preferences
}
```

#### Episodic Memory

```rust
pub struct EpisodicEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub actor: String,        // Who did it
    pub action: String,       // What happened
    pub content: String,      // Details
    pub importance: f32,      // 0.0-1.0 importance score
    pub embedding: Option<Vec<f32>>,
}

// Capacity-bounded with importance filtering
let memory = EpisodicMemory::with_capacity(1000);
memory.add(entry);
let recent = memory.recent(10);
let important = memory.important(0.7); // threshold
```

#### Semantic Memory

```rust
pub struct SemanticFact {
    pub id: String,
    pub subject: String,      // Entity
    pub predicate: String,    // Relationship
    pub object: String,       // Value
    pub confidence: f32,      // 0.0-1.0
    pub source: String,       // Where learned
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub categories: Vec<String>,
}

memory.add_fact(fact);
let facts = memory.by_category("finance");
let facts = memory.by_subject("customer-123");
```

#### Skill Memory

```rust
pub struct LearnedSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub proficiency: f32,     // 0.0-1.0 mastery level
    pub usage_count: u64,
    pub last_used: DateTime<Utc>,
    pub dependencies: Vec<String>,
    pub code_ref: Option<String>,
}

memory.learn(skill);           // Add/update skill
memory.record_usage("skill-1"); // Track usage
let expert = memory.by_proficiency(0.8); // High proficiency
```

### Export Formats

```rust
pub enum ExportFormat {
    Json,       // Human-readable
    Binary,     // MessagePack (compact)
    Encrypted,  // AES-256-GCM encrypted
}

let exporter = PassportExporter::new();
let bytes = exporter.export(&passport, &ExportOptions {
    format: ExportFormat::Json,
    compress: true,
    include_embeddings: false,
    allowed_regions: Some(vec!["EU".into()]),
    encryption_key: None,
})?;

// Convenience method
let json = exporter.export_json(&passport)?;
```

### GDPR Data Categories

```rust
pub enum DataCategory {
    Identity,       // Name, DID, identifiers
    Financial,      // Payment info, balances
    Communication,  // Messages, emails
    Behavioral,     // Actions, patterns
    Transactional,  // Purchase history
    Health,         // Health-related data
    Location,       // Geographic data
    AiGenerated,    // Predictions, inferences
    Other,
}
```

### GDPR Export

```rust
let exporter = GdprExporter::new();

let export = exporter.export(&passport)?;
// export.data_categories: Vec<DataCategory>
// export.processing_events: Vec<ProcessingEvent>
// export.summary: GdprSummary
// export.rights_info: RightsInfo

// Machine-readable JSON-LD format
let json_ld = exporter.to_json_ld(&passport)?;

// Human-readable text summary
let text = exporter.export_text(&passport)?;
```

### Processing Event Audit Trail

```rust
pub struct ProcessingEvent {
    pub timestamp: DateTime<Utc>,
    pub purpose: String,
    pub legal_basis: String,
    pub processor: String,
    pub categories: Vec<DataCategory>,
}
```

### Rights Information

```rust
pub struct RightsInfo {
    pub right_to_access: bool,
    pub right_to_rectification: bool,
    pub right_to_erasure: bool,
    pub right_to_restriction: bool,
    pub right_to_portability: bool,
    pub right_to_object: bool,
    pub contact_email: String,
    pub supervisory_authority: String,
}
```

### Passport Import

```rust
let importer = PassportImporter::new();

let result = importer.import(passport_bytes, &ImportOptions {
    verify_checksum: true,
    require_signature: false,
    allowed_regions: None,
    decryption_key: None,
})?;

// result.passport: MemoryPassport
// result.stats: ImportStats { memories_imported, facts_imported, skills_imported }
// result.warnings: Vec<String>
```

### Passport Merge

```rust
// Merge incoming passport into existing
importer.merge(&mut base_passport, &incoming_passport)?;
```

---

## 12. State Snapshots (Chain-Anchored)

Immutable, verifiable state backups with optional blockchain anchoring.

### Snapshot Status

```rust
pub enum SnapshotStatus {
    Creating,  // In progress
    Complete,  // Verified
    Anchored,  // On-chain
    Failed,
    Expired,
}
```

### Supported Chains

```rust
pub enum ChainType {
    Ethereum,
    Polygon,
    Near,
    Solana,
    InternalRaft, // No external chain
}
```

### Usage

```rust
let manager = SnapshotManager::new(
    SnapshotConfig::daily()
        .with_anchoring(ChainType::Polygon),
);

// Create snapshot
let snapshot = manager.create_snapshot("agent-123", state_bytes)?;

// Verify integrity
let valid = manager.verify(&snapshot)?;

// Restore
let data = manager.restore(snapshot.id)?;

// Anchor to blockchain
let anchor = manager.anchor_snapshot(snapshot.id)?;
// anchor.tx_hash, anchor.block_number
```

---

## 13. Global Mesh Sync

Multi-region CRDT synchronization with geo-fencing.

### Mesh Cell

```rust
pub struct MeshCell {
    pub id: String,
    pub region: DataRegion,
    pub endpoint: String,
    pub active: bool,
    pub last_heartbeat: u64,
}
```

### Data Regions

```rust
pub enum DataRegion {
    UsEast, UsWest,
    EuFrankfurt, EuIreland,
    AsiaSingapore, AsiaJapan,
    MenaRiyadh, MenaDubai,
    IndiaMumbai,
    Global,
}
```

### Geo-Fence Policy

| Region | Default Policy | Reference |
|--------|----------------|-----------|
| EU | Block PII export | GDPR |
| MENA | Block all export | PDPL |
| India | Block PII export | DPDP Act 2023 |
| US | Allow | — |

### Usage

```rust
let mesh = GlobalMesh::new("cell-eu-1".into(), DataRegion::EuFrankfurt);

// Register peers
mesh.register_cell(MeshCell {
    id: "cell-us-1".into(),
    region: DataRegion::UsEast,
    endpoint: "https://us-east.mesh.local".into(),
    active: true,
    last_heartbeat: 0,
}).await;

// Sync with geo-fence check
match mesh.sync_to_region("pii:user:123", DataRegion::UsEast, data).await {
    Ok(result) => println!("Synced to {} cells", result.cells_synced),
    Err(MeshError::GeoFenceBlocked { reason }) => {
        println!("Blocked: {}", reason);
    }
}
```

### Conflict Resolution

```rust
pub enum ConflictResolution {
    LastWriteWins,  // By timestamp
    FirstWriteWins, // Immutable
    Merge,          // For sets
    Custom,         // Custom resolver
}
```

---

## 14. Digital Twin Sandbox

Simulated environment for safe agent testing.

Per `FUTURE_INNOVATION_ROADMAP.md` Innovation #10.

### Sandbox Modes

```rust
pub enum SandboxMode {
    Mirror,   // Read-only mirror
    Clone,    // Full independent copy
    Isolated, // No external connections
    Chaos,    // Failure injection enabled
}
```

### Chaos Events

```rust
pub enum ChaosEventType {
    NetworkLatency,
    NetworkFailure,
    ServiceDown,
    RateLimit,
    MemoryPressure,
    DataCorruption,
    ClockSkew,
}
```

### Usage

```rust
let engine = SandboxEngine::new();

// Create snapshot
let snapshot = engine.snapshot("production");

// Create sandbox
let sandbox = engine.create_sandbox(
    "test-env",
    SandboxMode::Chaos,
    Some(&snapshot.id),
    Some(24), // 24-hour TTL
)?;

// Clone agent
engine.clone_agent(&sandbox.id, "agent-123", agent_state)?;

// Inject chaos
engine.inject_chaos(
    &sandbox.id,
    ChaosEventType::NetworkLatency,
    "api.external.com",
    5000, // 5s latency
)?;

// Run scenario
let result = engine.run_scenario(&sandbox.id, &scenario)?;
assert!(result.passed);
```

---

## 15. HTTP API (Server)

The Synapse server exposes REST endpoints for agent state management.

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/state/:agent_id` | Get agent state |
| `PUT` | `/state/:agent_id` | Update agent state |
| `GET` | `/intent/:agent_id` | Get current intent |
| `POST` | `/intent/:agent_id` | Start new intent |
| `POST` | `/intent/:agent_id/step` | Record intent step |
| `GET` | `/intent/:agent_id/drift` | Check for drift |

### Start Server

```bash
PORT=3002 cargo run --bin synapse-server
# 🧠 AgentKern-Synapse server running on http://0.0.0.0:3002
```

### Example: Intent Tracking Flow

```bash
# 1. Start intent
curl -X POST http://localhost:3002/intent/agent-123 \
  -H "Content-Type: application/json" \
  -d '{"intent": "Process purchase", "expected_steps": 5}'

# 2. Record steps
curl -X POST http://localhost:3002/intent/agent-123/step \
  -H "Content-Type: application/json" \
  -d '{"action": "validate_input", "result": "success"}'

# 3. Check drift
curl http://localhost:3002/intent/agent-123/drift
# {"drifted": false, "score": 20, "reason": null}
```

---

## 16. Complete Module Map

| Module | Lines | Purpose |
|--------|-------|---------|
| [`lib.rs`](../../packages/pillars/synapse/src/lib.rs) | 104 | Module exports |
| [`graph.rs`](../../packages/pillars/synapse/src/graph.rs) | 343 | Graph Vector Database |
| [`crdt.rs`](../../packages/pillars/synapse/src/crdt.rs) | 540 | CRDTs (GCounter, PNCounter, LWW, OR-Set) |
| [`intent.rs`](../../packages/pillars/synapse/src/intent.rs) | 184 | Intent path tracking |
| [`drift.rs`](../../packages/pillars/synapse/src/drift.rs) | 750 | Drift detection & alerting |
| [`state.rs`](../../packages/pillars/synapse/src/state.rs) | 237 | State store |
| [`types.rs`](../../packages/pillars/synapse/src/types.rs) | 105 | Core types |
| [`adaptive.rs`](../../packages/pillars/synapse/src/adaptive.rs) | 305 | Adaptive query execution |
| [`context_guard.rs`](../../packages/pillars/synapse/src/context_guard.rs) | 381 | RAG context protection |
| [`embeddings.rs`](../../packages/pillars/synapse/src/embeddings.rs) | 386 | Embedding configuration |
| [`encryption.rs`](../../packages/pillars/synapse/src/encryption.rs) | 511 | Envelope encryption |
| [`secure_passport.rs`](../../packages/pillars/synapse/src/secure_passport.rs) | 525 | Zero-Trust passports |
| [`state_snapshot.rs`](../../packages/pillars/synapse/src/state_snapshot.rs) | 501 | Chain-anchored snapshots |
| [`sandbox.rs`](../../packages/pillars/synapse/src/sandbox.rs) | 529 | Digital twin sandbox |
| [`mesh/mod.rs`](../../packages/pillars/synapse/src/mesh/mod.rs) | 226 | Global mesh controller |
| [`mesh/geo_fence.rs`](../../packages/pillars/synapse/src/mesh/geo_fence.rs) | 212 | Geo-fence policy |
| [`mesh/sync.rs`](../../packages/pillars/synapse/src/mesh/sync.rs) | 264 | CRDT sync protocol |
| [`passport/mod.rs`](../../packages/pillars/synapse/src/passport/mod.rs) | 24 | Memory passport exports |
| [`passport/export.rs`](../../packages/pillars/synapse/src/passport/export.rs) | ~250 | Passport export |
| [`passport/import.rs`](../../packages/pillars/synapse/src/passport/import.rs) | ~300 | Passport import |
| [`passport/gdpr.rs`](../../packages/pillars/synapse/src/passport/gdpr.rs) | ~350 | GDPR compliance |
| [`passport/layers.rs`](../../packages/pillars/synapse/src/passport/layers.rs) | ~350 | Memory hierarchy |
| [`passport/schema.rs`](../../packages/pillars/synapse/src/passport/schema.rs) | ~300 | Passport schema |
| [`polyglot/mod.rs`](../../packages/pillars/synapse/src/polyglot/mod.rs) | 259 | Language detection & memory |
| [`polyglot/embeddings.rs`](../../packages/pillars/synapse/src/polyglot/embeddings.rs) | ~170 | Polyglot embedder |
| [`bin/server.rs`](../../packages/pillars/synapse/src/bin/server.rs) | ~120 | HTTP server |

**Total: ~7,000+ lines of Rust**

---

## Key Design Decisions

### 1. Why CRDTs for Agent State?

| Traditional DB | CRDTs |
|----------------|-------|
| Requires coordination | Coordination-free |
| Network partition = failure | Partition-tolerant |
| Locking overhead | Lock-free |
| Eventual consistency via conflict | Eventual consistency guaranteed |

### 2. Why Graph + Vector (Not Pure Vector DB)?

- **Graph**: Captures relationships (Agent → Intent → Action → State)
- **Vector**: Enables semantic similarity search
- **Combined**: Rich context retrieval with relationship awareness

### 3. Why Field-Level Encryption?

| Full-Document | Field-Level |
|---------------|-------------|
| All or nothing access | Granular access control |
| Can't share subsets | Share only what's needed |
| No sensitivity levels | Multi-tier sensitivity |

### 4. Why Polyglot Embeddings?

| English-Only | Polyglot |
|--------------|----------|
| Poor Arabic/CJK results | Native model per language |
| One-size-fits-all | Region-optimal performance |
| US-centric | Global accessibility |

---

## Dependencies

```toml
[dependencies]
tokio = { version = "1.48", features = ["full"] }
polars = { version = "0.46", optional = true }
arrow = { version = "57", optional = true }
crdts-lib = { package = "crdts", version = "7", optional = true }
petgraph = { version = "0.6", optional = true }
axum = "0.8.8"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```

### Feature Flags

| Feature | Enables |
|---------|---------|
| `adaptive` | Polars/Arrow query execution |
| `crdts` | CRDT data structures |
| `graph` | Graph storage (petgraph) |
| `tee` | TEE integration |
| `full` | All features |

---

*Last updated: 2025-12-31*

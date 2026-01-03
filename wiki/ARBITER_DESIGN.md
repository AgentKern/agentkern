# Arbiter Pillar Design

> **The Conflict Resolution & Coordination Engine** — Thread-per-core runtime with Raft consensus, kill switch, and antifragile self-healing.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Thread-Per-Core Runtime (Hyper-Loop)](#2-thread-per-core-runtime-hyper-loop)
3. [Coordinator & Lock Manager](#3-coordinator--lock-manager)
4. [Raft Consensus (Atomic Business Locks)](#4-raft-consensus-atomic-business-locks)
5. [Kill Switch (Emergency Termination)](#5-kill-switch-emergency-termination)
6. [Antifragile Self-Healing Engine](#6-antifragile-self-healing-engine)
7. [Chaos Testing (Fault Injection)](#7-chaos-testing-fault-injection)
8. [Bulkhead Pattern (Agent Isolation)](#8-bulkhead-pattern-agent-isolation)
9. [Loop Prevention ($47K Incident)](#9-loop-prevention-47k-incident)
10. [Disaster Recovery Scheduler](#10-disaster-recovery-scheduler)
11. [Carbon-Aware Computing](#11-carbon-aware-computing)
12. [Cost Attribution Dashboard](#12-cost-attribution-dashboard)
13. [Human-in-the-Loop Escalation](#13-human-in-the-loop-escalation)
14. [Compliance Entities](#14-compliance-entities)
15. [Complete Module Map](#15-complete-module-map)

---

## 1. Overview

**Arbiter** is the conflict resolution and coordination engine that ensures agents operate safely, fairly, and within budget.

### What Arbiter Does

```
┌─────────────────────────────────────────────────────────────┐
│                      AgentKern-Arbiter                      │
├─────────────────────────────────────────────────────────────┤
│             Thread-per-Core Runtime (Hyper-Loop)            │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                 │
│  │ Core 0  │    │ Core 1  │    │ Core N  │                 │
│  │         │    │         │    │         │                 │
│  └────┬────┘    └────┬────┘    └────┬────┘                 │
│       │              │              │                       │
│       └──────────────┼──────────────┘                       │
│                      ▼                                      │
│           ┌─────────────────────┐                          │
│           │ Raft Lock Manager   │                          │
│           │ (Strong Consistency)│                          │
│           └─────────────────────┘                          │
│                      ▼                                      │
│           ┌─────────────────────┐                          │
│           │   Audit Ledger      │                          │
│           │ (ISO 42001 AIMS)    │                          │
│           └─────────────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

### Core Responsibilities

| Responsibility | Module | Description |
|----------------|--------|-------------|
| Lock Management | `coordinator.rs`, `locks.rs` | Atomic business locks with priority preemption |
| Consensus | `raft.rs` | Distributed consensus for strong consistency |
| Runtime | `thread_per_core.rs` | Sub-millisecond latency via core pinning |
| Emergency | `killswitch.rs` | Hardware-level agent termination |
| Resilience | `antifragile.rs`, `chaos.rs` | Self-healing and fault injection |
| Isolation | `bulkhead.rs` | Budget-based agent resource limits |
| Safety | `loop_prevention.rs` | Runaway loop detection |
| DR | `dr_scheduler.rs` | Automated disaster recovery drills |
| Sustainability | `carbon.rs` | Carbon-aware scheduling |
| FinOps | `cost.rs` | Per-agent cost tracking |
| Escalation | `escalation/` | Human-in-the-loop approval |
| Compliance | `entity/` | Shariah, screening, formation |

### Location

```
packages/pillars/arbiter/
├── src/
│   ├── lib.rs               # Module exports
│   ├── types.rs             # Core types (BusinessLock, CoordinationRequest)
│   ├── coordinator.rs       # High-level coordination API
│   ├── locks.rs             # Lock manager with TTL
│   ├── queue.rs             # Priority queue
│   ├── raft.rs              # Raft consensus
│   ├── thread_per_core.rs   # Hyper-loop runtime
│   ├── killswitch.rs        # Emergency termination
│   ├── antifragile.rs       # Self-healing engine
│   ├── chaos.rs             # Fault injection
│   ├── bulkhead.rs          # Resource isolation
│   ├── loop_prevention.rs   # $47K incident prevention
│   ├── dr_scheduler.rs      # DR drill automation
│   ├── carbon.rs            # Carbon scheduling
│   ├── cost.rs              # Cost tracking
│   ├── escalation/          # Human-in-the-loop
│   │   ├── mod.rs
│   │   ├── triggers.rs      # Escalation triggers
│   │   ├── approval.rs      # Approval workflows
│   │   └── webhook.rs       # Webhook notifications
│   ├── entity/              # Compliance entities
│   │   ├── mod.rs
│   │   ├── compliance.rs
│   │   ├── formation.rs
│   │   ├── liability.rs
│   │   ├── screening.rs
│   │   └── shariah.rs
│   └── bin/
│       └── server.rs        # HTTP server
└── tests/
```

---

## 2. Thread-Per-Core Runtime (Hyper-Loop)

Per `ARCHITECTURE.md` Section 1: "The Hyper-Loop"

### Why Thread-Per-Core?

| Traditional | Thread-Per-Core |
|-------------|-----------------|
| Thread pools with context switching | One thread per CPU core |
| Unpredictable latency spikes | Consistent sub-ms latency |
| Lock contention | Lock-free queues |
| Scheduler overhead | Dedicated work queues |

### Configuration

```rust
let config = ThreadPerCoreConfig {
    cores: 8,           // Number of cores to use
    pin_threads: true,  // Pin threads to cores (Linux only)
    queue_size: 1024,   // Work items per core queue
};

let runtime = ThreadPerCoreRuntime::new(config);
```

### Work Submission

```rust
// Submit to specific core
runtime.submit_to_core(0, Box::new(|| {
    // Work executed on Core 0
}))?;

// Submit with consistent hashing (same key → same core)
runtime.submit_hashed("customer:12345", Box::new(|| {
    // All requests for this customer go to same core
}))?;
```

### Thread Affinity (Linux)

Uses `libc::sched_setaffinity` to pin threads to specific CPU cores, eliminating scheduler-induced latency spikes.

---

## 3. Coordinator & Lock Manager

High-level coordination API combining locks and priority queues.

### Business Lock

```rust
pub struct BusinessLock {
    pub id: Uuid,
    pub resource: String,       // e.g., "customer:12345"
    pub locked_by: String,      // Agent ID
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub priority: i32,          // Higher = more important
    pub lock_type: LockType,    // Read, Write, Exclusive
}
```

### Lock Types

| Type | Description | Multiple Holders |
|------|-------------|------------------|
| `Read` | Shared read access | ✅ Yes |
| `Write` | Exclusive write access | ❌ No |
| `Exclusive` | No other access | ❌ No |

### Coordination Request

```rust
let request = CoordinationRequest::new("agent-1", "database:accounts")
    .with_operation(LockType::Exclusive)
    .with_priority(10)
    .with_duration_ms(60000);

let result = coordinator.request(request).await;
match result {
    CoordinationResult { granted: true, lock: Some(lock), .. } => {
        // Lock acquired
    }
    CoordinationResult { queue_position: Some(pos), .. } => {
        println!("Queued at position {}", pos);
    }
    CoordinationResult { reason: Some(r), .. } => {
        println!("Denied: {}", r);
    }
}
```

### Priority Preemption

Higher priority agents can preempt locks held by lower priority agents:

```rust
// Low priority agent holds lock
manager.acquire("agent-1", "resource", 5, LockType::Write, None).await?;

// High priority agent preempts
let lock = manager.acquire("agent-2", "resource", 10, LockType::Write, None).await?;
assert_eq!(lock.locked_by, "agent-2");  // Preempted!
```

---

## 4. Raft Consensus (Atomic Business Locks)

Per `ARCHITECTURE.md` Section 3: "The Speed of Light"

> Used **only** for strong consistency operations (e.g., spending money).

### Lock Commands

```rust
pub enum LockCommand {
    Acquire { resource: String, agent_id: String, priority: i32, ttl_ms: u64 },
    Release { resource: String, agent_id: String },
    Heartbeat { resource: String, agent_id: String },
}
```

### Raft State Machine

```rust
let mut manager = RaftLockManager::new(RaftConfig::default());

// Become leader (single-node or after election)
manager.become_leader();

// Acquire through consensus
let log_index = manager.acquire_lock("payment:123", "agent-1", 5, 30000)?;

// Release through consensus
manager.release_lock("payment:123", "agent-1")?;
```

### Configuration

```rust
pub struct RaftConfig {
    pub node_id: String,
    pub heartbeat_interval_ms: u64,  // Default: 150ms
    pub election_timeout_ms: u64,    // Default: 300ms
}
```

---

## 5. Kill Switch (Emergency Termination)

Per `EXECUTION_MANDATE.md` §6: "Hardware-level Red Button to terminate rogue agent swarms"

### Kill Reasons

```rust
pub enum KillReason {
    PolicyViolation,        // Policy rule breached
    BudgetExhausted,       // Budget exceeded
    UnauthorizedAccess,    // Permission denied
    AnomalyDetected,       // Behavioral anomaly
    ManualTermination,     // Human-initiated
    EmergencyShutdown,     // System-wide emergency
    TimeoutExceeded,       // Time limit breached
    ParentTerminated,      // Parent agent died
    Custom(String),
}
```

### Termination Types

| Type | Description | Cleanup |
|------|-------------|---------|
| `Graceful` | Allow cleanup | ✅ Yes |
| `Forced` | Immediate stop | ⚠️ Partial |
| `HardwareKill` | No cleanup | ❌ No |

### Usage

```rust
let killswitch = KillSwitch::new();

// Terminate single agent
let record = killswitch.terminate_agent(
    "agent-123",
    KillReason::BudgetExhausted,
    TerminationType::Graceful,
    Some("ops-team".into()),
);

// Terminate entire swarm
killswitch.terminate_swarm(
    "swarm-456",
    KillReason::PolicyViolation,
    TerminationType::Forced,
    None,
);

// EMERGENCY: Global shutdown
killswitch.emergency_shutdown(Some("security-team".into()));

// Check if agent is alive
if !killswitch.is_agent_alive("agent-123") {
    // Agent has been terminated
}
```

---

## 6. Antifragile Self-Healing Engine

Per Roadmap: "Anti-Fragile Self-Healing Engine - make agents stronger through failure"

Inspired by Nassim Taleb's "Antifragile" (2012) — systems that gain from disorder.

### Failure Classification

```rust
pub enum FailureClass {
    NetworkTimeout,
    ResourceExhaustion,
    ServiceUnavailable,
    ValidationError,
    PolicyViolation,
    Unknown,
}
```

### Adaptation Rate (EPISTEMIC WARRANT)

| Parameter | Default | Rationale |
|-----------|---------|-----------|
| Base rate | 1.0 | Neutral starting point |
| Boost factor | 1.5x | Faster response to failures |
| Decay factor | 0.9x | Slower return to baseline |
| Max rate | 10.0 | Upper bound |
| Min rate | 0.1 | Lower bound |

The boost/decay asymmetry ensures faster response to failures than return to baseline after recovery.

### Recovery Strategies

```rust
pub enum RecoveryStrategyType {
    Retry,              // Simple retry
    ExponentialBackoff, // Exponential backoff
    Fallback,           // Use fallback service
    CircuitBreaker,     // Open circuit
    GracefulDegradation,// Reduce functionality
    Escalate,           // Human intervention
    Quarantine,         // Isolate agent
}
```

### Circuit Breaker

```rust
let engine = AntifragileEngine::new();

// Record failure
let failure = Failure::new("payment-service", "Connection refused")
    .with_context("endpoint", "api.payments.com");
    
engine.record_failure(failure);

// Check circuit state
match engine.circuit_state("payment-service") {
    CircuitState::Closed => { /* Normal operation */ }
    CircuitState::Open => { /* Fast-fail requests */ }
    CircuitState::HalfOpen => { /* Allow probe requests */ }
}
```

---

## 7. Chaos Testing (Fault Injection)

Per `MANDATE.md`: "Antifragile by default - every failure makes us stronger"

### Chaos Errors

```rust
pub enum ChaosError {
    Timeout,
    NetworkError,
    ServiceUnavailable,
    InternalError,
    DataCorruption,
    RateLimitExceeded,
    AuthFailure,
}
```

### Configuration Presets

| Preset | Failure Rate | Latency | Description |
|--------|--------------|---------|-------------|
| `mild()` | 1% | 0-50ms | Light testing |
| `moderate()` | 5% | 0-200ms | Regular testing |
| `extreme()` | 20% | 0-1000ms | Stress testing |

### Usage

```rust
let monkey = ChaosMonkey::new(ChaosConfig::moderate());

// Maybe inject chaos
let result = monkey.maybe_inject(|| {
    call_external_service()
});

match result {
    ChaosResult::Ok(v) => { /* Normal execution */ }
    ChaosResult::Delayed { result, delay_ms } => { 
        println!("Delayed by {}ms", delay_ms);
    }
    ChaosResult::Error(e) => {
        println!("Chaos injected: {:?}", e);
    }
}

// Get statistics
let stats = monkey.stats();
println!("Chaos rate: {:.2}%", stats.chaos_rate() * 100.0);
```

---

## 8. Bulkhead Pattern (Agent Isolation)

Per Antifragility Report: "Agent Sandbox - Bulkhead pattern"

### Resource Quotas

```rust
pub enum ResourceQuota {
    ApiCalls { limit: u64, current: u64 },
    Memory { limit_bytes: u64 },
    CpuTime { limit_ms: u64 },
    Tokens { limit: u64, current: u64 },
    Cost { limit_micros: u64, current: u64 }, // $1 = 1_000_000 microdollars
}
```

### Tier Presets

| Tier | Concurrent | API Calls/hr | Tokens | Cost |
|------|------------|--------------|--------|------|
| `basic()` | 5 | 100 | 10K | $1 |
| `premium()` | 20 | 1000 | 100K | $10 |
| `enterprise()` | 100 | 10000 | 1M | $100 |

### Usage

```rust
let bulkhead = Bulkhead::new("agent-123", BulkheadConfig::premium());

// Acquire permit (blocks if at capacity)
let permit = bulkhead.acquire().await?;

// Do work...
do_api_call();

// Permit released on drop

// Check consumption
bulkhead.consume_tokens(500)?;
bulkhead.consume_cost(0.05)?;

// Get statistics
let stats = bulkhead.stats();
```

### Rejection Reasons

```rust
pub enum BulkheadRejection {
    MaxConcurrentExceeded { current: usize, max: usize },
    QuotaExceeded { quota_type: String, current: u64, limit: u64 },
    AgentSuspended { reason: String },
    Timeout { waited_ms: u64 },
}
```

---

## 9. Loop Prevention ($47K Incident)

Prevents the infamous **$47,000 Runaway AI Loop incident** by detecting and stopping infinite agent-to-agent message loops.

### Detection Methods

1. **Hop Count**: Max hops per message (default: 10)
2. **Cycle Detection**: Visited agent appears twice
3. **Pair Rate Limiting**: Max messages between two agents per minute
4. **Cost Accumulation**: Total cost across all hops

### Configuration

```rust
pub struct LoopPreventionConfig {
    pub max_hops: u32,           // Default: 10
    pub max_pair_rate: u32,      // Default: 50/min
    pub cost_limit: f64,         // Default: $100
    pub window_seconds: u64,     // Default: 60
}
```

### Presets

| Preset | Max Hops | Pair Rate | Cost Limit |
|--------|----------|-----------|------------|
| `default()` | 10 | 50/min | $100 |
| `strict()` | 5 | 20/min | $10 |
| `relaxed()` | 20 | 100/min | $1000 |

### Usage

```rust
let preventer = LoopPreventer::new(LoopPreventionConfig::strict());

let mut message = TrackedMessage::new("msg-123", "agent-A");
message.add_hop("agent-B", 0.05);
message.add_hop("agent-C", 0.10);

// Check before forwarding
match preventer.check(&message) {
    Ok(()) => { /* Forward message */ }
    Err(LoopPreventionError::CycleDetected { .. }) => {
        // Same agent visited twice
    }
    Err(LoopPreventionError::MaxHopsExceeded { .. }) => {
        // Too many hops
    }
    Err(LoopPreventionError::CostLimitExceeded { .. }) => {
        // $47K prevention!
    }
}
```

---

## 10. Disaster Recovery Scheduler

Per Antifragility Roadmap: "Automated DR Drills"

### Drill Types

```rust
pub enum DrillType {
    FullFailover,       // Complete region failover
    DatabaseFailover,   // DB switchover
    ServiceRestart,     // Service restart under load
    NetworkPartition,   // Network split simulation
    FullChaos,          // All failure modes
}
```

### Schedule Monthly Drills

```rust
let scheduler = DRScheduler::staging();

// Schedule for first of each month at 03:00 UTC
let drill_id = scheduler.schedule_monthly_drill(
    "staging",
    "us-east-1",
    DrillType::DatabaseFailover,
);
```

### Run Drill Immediately

```rust
let result = scheduler.run_drill(DrillType::FullFailover, "staging");

println!("Success: {}", result.success);
println!("Duration: {}ms", result.duration_ms);
println!("RTO: {:?}s", result.rto_seconds);

// Metrics collected
println!("Failover time: {}s", result.metrics.failover_time_secs);
println!("Data loss: {} records", result.metrics.data_loss_count);
println!("Service impact: {}s", result.metrics.service_impact_secs);
```

### Calculate RTO

```rust
// Get average Recovery Time Objective from history
let avg_rto = scheduler.average_rto_secs();
```

---

## 11. Carbon-Aware Computing

Per `EXECUTION_MANDATE.md` §7: "Carbon-Aware & Sustainable Computing"

### Carbon Intensity Levels

```rust
pub enum CarbonIntensity {
    Green,   // < 100 gCO2eq/kWh (hydro, nuclear, solar)
    Low,     // 100-300 gCO2eq/kWh
    Medium,  // 300-500 gCO2eq/kWh
    High,    // > 500 gCO2eq/kWh (coal)
}
```

### GHG Protocol Scopes

| Scope | Description |
|-------|-------------|
| Scope 1 | Direct emissions |
| Scope 2 | Indirect (electricity) |
| Scope 3 | Value chain |

### Usage

```rust
let scheduler = CarbonScheduler::new();

// Get green regions
let green = scheduler.green_regions();
// [Quebec, Iceland, Norway, Sweden, ...]

// Select greenest from candidates
let best = scheduler.select_greenest(&["us-east-1", "eu-north-1", "ca-central-1"]);
// "ca-central-1" (Quebec hydro)

// Calculate emissions
let grams = scheduler.calculate_emissions("us-east-1", 1.5)?; // 1.5 kWh
// 600 gCO2eq

// Record and track
let record = scheduler.record_transaction(
    "txn-123".into(),
    "eu-north-1",
    0.5, // kWh
)?;

// Carbon savings vs average
let savings = scheduler.carbon_savings("eu-north-1", 1.0);
// 350 gCO2eq saved
```

### Default Region Data

| Region | gCO2eq/kWh | Source |
|--------|------------|--------|
| ca-central-1 (Quebec) | 30 | Hydro |
| eu-north-1 (Stockholm) | 50 | Hydro/Nuclear |
| us-east-1 (Virginia) | 400 | Mixed |
| ap-south-1 (Mumbai) | 700 | Coal |

---

## 12. Cost Attribution Dashboard

Per `MANDATE.md` Section 1: Autonomous agents need spending controls.

### Cost Categories

```rust
pub enum CostCategory {
    LlmTokens,      // LLM API costs
    Storage,        // Data storage
    Compute,        // CPU/GPU
    Network,        // Egress
    ExternalApi,    // Third-party APIs
    CarbonOffset,   // Green premium
    Custom(String),
}
```

### Track Costs

```rust
let tracker = CostTracker::new();

// Record event
let event = tracker.event("agent-123", CostCategory::LlmTokens)
    .resource("gpt-4")
    .amount(0.03)
    .quantity(1000.0, "tokens")
    .task("task-456")
    .build();

let alert = tracker.record(event);

if let Some(alert) = alert {
    if alert.level.should_pause() {
        // Pause agent execution
    }
}
```

### Thresholds & Alerts

```rust
tracker.add_threshold(CostThreshold {
    agent_pattern: Some("agent-*".into()),
    category: Some(CostCategory::LlmTokens),
    amount_usd: 10.0,
    period_seconds: 3600,
    level: AlertLevel::Warning,
});

tracker.add_threshold(CostThreshold {
    agent_pattern: None, // All agents
    category: None,      // All categories
    amount_usd: 100.0,
    period_seconds: 86400,
    level: AlertLevel::Critical,
});
```

### Export

```rust
let csv = tracker.export_csv();
// agent_id,category,resource,amount_usd,quantity,unit,timestamp
```

---

## 13. Human-in-the-Loop Escalation

Per `MANDATE.md` Section 6: Autonomous Agent Security

### Escalation Levels

```rust
pub enum EscalationLevel {
    Low,       // Notification only
    Medium,    // Requires acknowledgment
    High,      // Requires approval
    Critical,  // Immediate intervention
}
```

| Level | Timeout | Pauses Agent |
|-------|---------|--------------|
| Low | 1 hour | ❌ No |
| Medium | 30 min | ❌ No |
| High | 10 min | ✅ Yes |
| Critical | 5 min | ✅ Yes |

### Trigger Types

```rust
pub enum TriggerType {
    TrustScore,         // Trust below threshold
    BudgetExceeded,     // Over budget
    HighRiskAction,     // Risky operation
    PolicyViolation,    // Rule breached
    ErrorRateExceeded,  // Too many errors
    Manual,             // Human-initiated
    Custom(String),
}
```

### Usage

```rust
let trigger = EscalationTrigger::new(TriggerConfig::default());

// Evaluate trust score
if let Some(result) = trigger.evaluate_trust("agent-123", 0.3) {
    match result.level {
        EscalationLevel::Critical => {
            // Pause agent, notify security team
        }
        _ => {}
    }
}

// Evaluate budget
if let Some(result) = trigger.evaluate_budget("agent-123", 95.0, 100.0) {
    // 95% of budget used
}

// Manual escalation
let result = trigger.manual_escalate(
    "agent-123",
    "Unusual behavior pattern",
    EscalationLevel::High,
);
```

### Webhook Notifications

```rust
let notifier = WebhookNotifier::new();

notifier.add_webhook(WebhookConfig {
    url: "https://slack.com/webhook/xxx".into(),
    secret: Some("hmac-secret".into()),
    min_level: EscalationLevel::Medium,
    timeout_ms: 5000,
});

notifier.notify(&escalation_result).await?;
```

### Approval Workflow

```rust
let workflow = ApprovalWorkflow::new();

// Create approval request
let request = workflow.create_request(
    "agent-123",
    "deploy-production",
    EscalationLevel::High,
);

// Submit for approval
workflow.submit(request)?;

// Approve/reject
workflow.decide(request.id, ApprovalDecision::Approve, "admin")?;

// Check status
match workflow.get_status(request.id)? {
    ApprovalStatus::Approved => { /* Proceed */ }
    ApprovalStatus::Rejected => { /* Abort */ }
    ApprovalStatus::Pending => { /* Wait */ }
    ApprovalStatus::Expired => { /* Timeout */ }
}
```

---

## 14. Compliance Entities

Global entity formation, liability protection, and multi-framework compliance.

### Entity Types

```rust
pub enum EntityType {
    Llc,           // US/EU/UK/SG/AE
    Corporation,   // US/EU/UK/JP/CN
    Takaful,       // MY/SA/AE/BH/PK/ID - Islamic mutual risk sharing
    Waqf,          // MY/SA/AE/TR/PK - Islamic endowment/trust
    Dao,           // WY/TN/UT/CH - Blockchain-native
    Partnership,   // US/EU/UK
    Individual,    // Global
}
```

| Type | Shariah-Compliant | Key Jurisdictions |
|------|-------------------|-------------------|
| Takaful | ✅ By default | Malaysia, Saudi, UAE, Bahrain |
| Waqf | ✅ By default | Malaysia, Saudi, Turkey |
| DAO | ✅ Compatible | Wyoming, Tennessee, Switzerland |
| LLC | ⚠️ Configurable | US, EU, UK, Singapore |

### Liability Models

```rust
pub enum LiabilityModel {
    CorporateLiability,     // Traditional LLC/Corp protection
    TakafulMutual,          // Islamic mutual pooling (Shariah ✅)
    ConventionalInsurance,  // Traditional (Contains riba ❌)
    DaoTreasury,            // Smart contract reserves (Shariah ✅)
    SelfInsured,            // Agent holds reserves (Shariah ✅)
    None,                   // Full personal liability
}
```

```rust
// Create Takaful protection
let protection = LiabilityProtection::takaful(
    "pool-001",
    1_000_000,  // SAR 1M limit
    "SAR",
);

// Coverage types: General, Professional, Cyber, D&O, Product
```

### Shariah Compliance Checker

Implements Islamic finance principles per AAOIFI standards:

| Check | Description | Severity |
|-------|-------------|----------|
| **Riba** | No interest/usury | Major (1.0) |
| **Gharar** | No excessive uncertainty | Variable |
| **Maysir** | No gambling/speculation | Major (0.9) |
| **HalalSector** | No prohibited sectors | Major (1.0) |
| **Purification** | Income cleansing | Advisory |
| **Zakat** | Charitable obligation | Advisory |

```rust
let checker = ShariahCompliance::new()
    .with_strictness(StrictnessLevel::Moderate); // AAOIFI standards

let tx = TransactionCheck {
    transaction_type: TransactionType::Investment,
    amount: 50000,
    currency: "SAR".into(),
    sector: Some("technology".into()),
    interest_rate: None,        // No riba
    uncertainty_level: Some(0.1), // Low gharar
};

let result = checker.check_transaction(&tx);
assert!(result.compliant);
```

**Strictness Levels:**

| Level | Gharar Threshold | Reference |
|-------|------------------|-----------|
| Lenient | 0.8 | Mainstream scholars |
| Moderate | 0.6 | AAOIFI standards |
| Strict | 0.4 | Conservative interpretation |

**Prohibited Sectors:** alcohol, tobacco, gambling, pork, adult_entertainment, weapons, conventional_finance

### Investment & Product Screening

Multi-framework screening beyond Islamic finance:

```rust
pub enum ScreeningCriteria {
    Halal,          // Islamic (no haram sectors)
    Kosher,         // Jewish (no pork, shellfish)
    Hindu,          // No beef
    Vegan,          // No animal products
    Vegetarian,     // No meat
    Organic,        // Organic certification
    Environmental,  // ESG - E score ≥ 50
    Social,         // ESG - S score ≥ 50
    Governance,     // ESG - G score ≥ 50
}
```

**ESG Threshold Rationale (EPISTEMIC WARRANT):**

| Our Score | MSCI Equivalent | Rating |
|-----------|-----------------|--------|
| 70-100 | 7.0-10.0 | AAA/AA (Leaders) |
| 30-70 | 3.0-7.0 | A/BBB/BB (Average) |
| 0-30 | 0.0-3.0 | B/CCC (Laggards) |

Threshold of **50.0** = minimum "Average" rating, acceptable for non-specialist ESG funds.

Reference: [MSCI ESG Ratings Methodology (2024)](https://www.msci.com/esg-ratings)

```rust
// Halal screening
let screener = InvestmentScreener::halal();

// Full ESG screening
let screener = InvestmentScreener::esg();

// Multi-criteria
let screener = InvestmentScreener::new(vec![
    ScreeningCriteria::Halal,
    ScreeningCriteria::Environmental,
]);

let target = ScreenTarget {
    name: "Green Energy Fund".into(),
    sectors: vec!["renewable_energy".into()],
    esg_scores: Some(EsgScores {
        environmental: 85.0,
        social: 70.0,
        governance: 75.0,
    }),
};

let result = screener.screen(&target);
assert!(result.approved);
assert_eq!(result.score, 100.0);
```

### Cultural & Ethical Compliance

Beyond finance — applies to **all agent operations**:

| Framework | Primary Regions | Example Checks |
|-----------|-----------------|----------------|
| Islamic | MENA, SEA | No haram imagery, prayer times |
| Jewish | IL, US | Sabbath scheduling, kosher supply |
| Christian | Latin America, Southern US | Content moderation |
| Secular | Global | ESG, vegan, ethical supply |
| ChildProtection | US, EU | COPPA, KOSA compliance |
| Accessibility | Global | ADA, WCAG compliance |

```rust
// Auto-detect frameworks for region
let checker = CulturalCompliance::for_region("SA");
// Includes: Islamic (primary)

let checker = CulturalCompliance::for_region("US");
// Includes: Secular, ChildProtection, Accessibility

// Check any operation
let operation = OperationCheck {
    operation_type: OperationType::Content,
    context: OperationContext {
        region: "MY".into(),
        audience: Some(AudienceType::General),
        tags: vec!["food".into(), "recipe".into()],
        ..Default::default()
    },
};

let result = checker.check(&operation);
```

**Operation Types:**
- `Content` — Generation/display
- `SupplyChain` — Product sourcing
- `ScheduledTask` — Time-sensitive operations (prayer times, Sabbath)
- `Communication` — Outreach/marketing
- `DataProcessing` — Privacy compliance
- `Financial` — Transaction compliance

---

## 15. Complete Module Map

| Module | Lines | Purpose |
|--------|-------|---------|
| [`lib.rs`](../../packages/pillars/arbiter/src/lib.rs) | 107 | Module exports |
| [`types.rs`](../../packages/pillars/arbiter/src/types.rs) | 177 | Core types (BusinessLock, CoordinationRequest) |
| [`coordinator.rs`](../../packages/pillars/arbiter/src/coordinator.rs) | 171 | High-level coordination API |
| [`locks.rs`](../../packages/pillars/arbiter/src/locks.rs) | 231 | Lock manager with TTL |
| [`queue.rs`](../../packages/pillars/arbiter/src/queue.rs) | ~170 | Priority queue |
| [`raft.rs`](../../packages/pillars/arbiter/src/raft.rs) | 359 | Raft consensus |
| [`thread_per_core.rs`](../../packages/pillars/arbiter/src/thread_per_core.rs) | 229 | Hyper-loop runtime |
| [`killswitch.rs`](../../packages/pillars/arbiter/src/killswitch.rs) | 345 | Emergency termination |
| [`antifragile.rs`](../../packages/pillars/arbiter/src/antifragile.rs) | 938 | Self-healing engine |
| [`chaos.rs`](../../packages/pillars/arbiter/src/chaos.rs) | 544 | Fault injection |
| [`bulkhead.rs`](../../packages/pillars/arbiter/src/bulkhead.rs) | 593 | Resource isolation |
| [`loop_prevention.rs`](../../packages/pillars/arbiter/src/loop_prevention.rs) | 450 | $47K prevention |
| [`dr_scheduler.rs`](../../packages/pillars/arbiter/src/dr_scheduler.rs) | 420 | DR drill automation |
| [`carbon.rs`](../../packages/pillars/arbiter/src/carbon.rs) | 363 | Carbon scheduling |
| [`cost.rs`](../../packages/pillars/arbiter/src/cost.rs) | 526 | Cost tracking |
| [`escalation/mod.rs`](../../packages/pillars/arbiter/src/escalation/mod.rs) | 19 | Escalation exports |
| [`escalation/triggers.rs`](../../packages/pillars/arbiter/src/escalation/triggers.rs) | 377 | Escalation triggers |
| [`escalation/approval.rs`](../../packages/pillars/arbiter/src/escalation/approval.rs) | ~350 | Approval workflows |
| [`escalation/webhook.rs`](../../packages/pillars/arbiter/src/escalation/webhook.rs) | ~370 | Webhook notifications |
| [`entity/mod.rs`](../../packages/pillars/arbiter/src/entity/mod.rs) | ~24 | Entity exports |
| [`entity/compliance.rs`](../../packages/pillars/arbiter/src/entity/compliance.rs) | ~300 | Compliance checks |
| [`entity/formation.rs`](../../packages/pillars/arbiter/src/entity/formation.rs) | ~135 | Entity formation |
| [`entity/liability.rs`](../../packages/pillars/arbiter/src/entity/liability.rs) | ~120 | Liability tracking |
| [`entity/screening.rs`](../../packages/pillars/arbiter/src/entity/screening.rs) | ~190 | Sanctions screening |
| [`entity/shariah.rs`](../../packages/pillars/arbiter/src/entity/shariah.rs) | ~245 | Shariah compliance |
| [`bin/server.rs`](../../packages/pillars/arbiter/src/bin/server.rs) | ~120 | HTTP server |

**Total: ~7,500+ lines of Rust**

---

## Key Design Decisions

### 1. Why Thread-Per-Core?

| Concern | Solution |
|---------|----------|
| Context switching latency | Pinned threads, no switching |
| Scheduler jitter | Dedicated work queues |
| Cache thrashing | Core affinity |
| Lock contention | Lock-free message passing |

### 2. Why Raft for Locks?

| CAP Trade-off | Arbiter Choice |
|---------------|----------------|
| **Financial transactions** | Strong consistency (Raft) |
| **State queries** | Eventual consistency (CRDTs in Synapse) |
| **Policy decisions** | Symbolic + Neural (Gate) |

### 3. Why Antifragile vs Just Resilient?

| Resilience | Antifragility |
|------------|---------------|
| Survives stress | **Gains** from stress |
| Returns to baseline | Improves beyond baseline |
| Passive recovery | Active adaptation |
| Static thresholds | Dynamic learning |

---

## Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
openraft = { version = "0.9", optional = true }
axum = "0.8.8"
agentkern-governance = { path = "../governance" }
```

### Feature Flags

| Feature | Enables |
|---------|---------|
| `raft` | OpenRaft consensus |
| `thread_per_core` | tokio-uring (Linux) |
| `full` | All features |

---

*Last updated: 2025-12-31*

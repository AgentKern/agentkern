# AgentKern Automated Runbooks

**Status**: Active  
**Executor**: Autonomous (Synapse MeshOrchestrator / Arbiter Antifragile)

## Overview
These runbooks define the *autonomous* actions taken by the AgentKern kernel in response to specific failure signals. Manual intervention is only required if autonomous recovery fails > 3 times.

## 1. Mesh Failures (Synapse)

### ERR-MESH-001: Region Health Critical
- **Symptom**: `SemanticHealthReport` returns `HealthStatus::Critical` (e.g. power outage, natural disaster).
- **Detection**: `MeshOrchestrator::check_and_migrate` loop.
- **Automated Action**:
    1.  Identify healthiest alternative region (lowest carbon/latency).
    2.  Trigger `MigrationManager::hibernate`.
    3.  Move agent state to target region.
    4.  Resume execution.
- **Manual Override**: `synapsectl force-migrate <agent_id> <region>`

### ERR-MESH-002: Region Health Degraded
- **Symptom**: `SemanticHealthReport` returns `HealthStatus::Degraded` (e.g. memory leak, high latency).
- **Detection**: `MeshOrchestrator::check_and_migrate` loop.
- **Automated Action**:
    1.  **Self-Healing**: Trigger `heal_local_agent`.
    2.  Hibernate state to local persistence.
    3.  Restart microVM / reload state.
    4.  Verify health.
    5.  If health remains degraded > 3 attempts, escalate to ERR-MESH-001 (Migrate).

## 2. Inter-Agent Failures (Nexus/Arbiter)

### ERR-NET-001: Protocol Timeout
- **Symptom**: `ChaosProxy` or network layer reports timeout > 5000ms.
- **Automated Action**:
    1.  **Arbiter**: Circuit breaker opens for target agent.
    2.  **Nexus**: Retry with exponential backoff (up to 3x).
    3.  If persistent, route task to alternative agent with same skills.

### ERR-SEC-001: Intent Drift
- **Symptom**: `DriftDetector` score > 70.
- **Automated Action**:
    1.  **Arbiter**: Deny coordination request.
    2.  **Gate**: Lock agent capabilities.
    3.  Notify human operator (Manual Intervention Required).

## 3. Resource Failures (Arbiter)

### ERR-LOCK-001: Lock Contention
- **Symptom**: `CoordinationResult::Queued`.
- **Automated Action**:
    1.  Enqueue request.
    2.  Wait for estimated duration.
    3.  If priority is Critical (90+), preempt current owner (kill/revoke).

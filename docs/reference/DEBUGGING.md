# Failure Modes & Debugging

This document maps system errors to resolution steps for the AgentKern Rust Runtime.

---

## 1. Safety & Policy Failures (Gate)

| Error Code | Meaning | Resolution |
|------------|---------|------------|
| `POLICY_VIOLATION` | Action blocked by symbolic rule. | Check `SymbolicRiskScore` in logs. Adjust `.policy.yaml`. |
| `NEURAL_BLOCK` | Malicious intent detected by ONNX model. | Inspect `NeuralRiskScore`. False positive? Add whitelist rule. |
| `LATENCY_TIMEOUT` | Verify hook took >20ms. | Ensure neural model is quantized (INT8). |

## 2. Infrastructure Failures (Core)

| Error Code | Meaning | Resolution |
|------------|---------|------------|
| `TEE_SIG_FAIL` | Process not running in enclave. | Set `environment: local` or check `/dev/tdx_guest`. |
| `DB_POOL_EXHAUSTED` | PostgreSQL connection limits. | Increase `max_connections` or reduce concurrency. |
| `IO_URING_FAIL` | Kernel incompatibility (Linux <5.10). | Upgrade kernel or disable the `io_uring` feature flag. |

## 3. Protocol Failures (Nexus)

| Error Code | Meaning | Resolution |
|------------|---------|------------|
| `SCHEMA_MISMATCH` | Input JSON != Adapter Spec. | Validate against `MCP` or `A2A` schema definitions. |
| `WASM_PANIC` | Adapter crashed in sandbox. | Check adapter logs for memory violations. |

# Contributing to AgentKern

## 📐 Engineering Standards

### 1. Naming Conventions

**Type Naming** in Rust/TypeScript:
- Request/Response: `*Request` and `*Result`.
    - `GateVerifyRequest` -> `GateVerifyResult`.
    - Native bindings: `NativeVerifyRequest`.

**Field Naming**:
- **Risk Scores**: `final_risk_score` (0-100).
- **Carbon**: `total_co2_grams`, `daily_limit_grams`.
- **Time**: `created_at` (ISO), `latency_ms` (duration).
- **IDs**: `agent_id`, `request_id`.

### 2. Module Structure

| Component | Format | Example |
|-----------|--------|---------|
| Crate | `agentkern-*` | `agentkern-gate` |
| Module | `snake_case` | `shariah_compliance` |
| Struct | `PascalCase` | `GateEngine` |
| URL Path | `kebab-case` | `/v1/gate/verify-action` |

### 3. Protocol Enums

| Protocol | Variant | Serde Value |
|----------|---------|-------------|
| Native | `AgentKern` | `agentkern` |
| Google A2A | `GoogleA2A` | `a2a` |
| MCP | `AnthropicMCP` | `mcp` |

---

## 🛠️ Development Workflow

1. **Feature Flags**: Always enable `full` features for local testing.
2. **Docs**: Update `docs/` if you change public APIs.
3. **Tests**: Run `cargo test --workspace` before pushing.
4. **CI Policy**: Follow `docs/governance/CI_POLICY.md` when proposing workflow gate changes.

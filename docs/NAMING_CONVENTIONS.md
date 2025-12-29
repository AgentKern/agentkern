# AgentKern Naming Conventions

This document establishes naming standards for the AgentKern codebase.

## Type Naming

### Request/Response Pattern
Use `*Request` and `*Result` (not `*Response`) for all API operations:

| Package | Request | Result |
|---------|---------|--------|
| Gate | `VerificationRequest` | `VerificationResult` |
| Native | `VerifyRequest` → **`NativeVerifyRequest`** | `VerifyResult` → **`NativeVerifyResult`** |
| SSO | `TokenExchangeRequest` | `SsoResult` |
| Nexus | `RouteRequest` | `RouteResult` |

> **Rationale**: Native bindings use shorter names with `Native` prefix to distinguish from core types.

## Field Naming

### Risk Scores
- `final_risk_score` - Combined score used for decisions (0-100)
- `symbolic_risk_score` - Score from policy evaluation (0-100)
- `neural_risk_score` - Score from ML model (0-100, optional)
- `risk_score` - Generic shorthand when context is clear

### Carbon/Sustainability
- `*_co2_grams` - CO2 amounts (e.g., `total_co2_grams`)
- `*_limit_grams` - Budget limits (e.g., `daily_limit_grams`)
- `*_kwh` - Energy amounts (e.g., `total_energy_kwh`)

### Identifiers
- `*_id` - Unique identifiers (e.g., `agent_id`, `request_id`)
- `*_ids` - Collections of IDs

### Timestamps
- `*_at` - Instant in time (e.g., `created_at`, `updated_at`)
- `*_ms` - Duration in milliseconds (e.g., `latency_ms`, `timeout_ms`)

## Module Naming

| Type | Convention | Example |
|------|------------|---------|
| Crate | `agentkern-*` | `agentkern-gate` |
| Module | `snake_case` | `crypto_agility` |
| Struct | `PascalCase` | `GateEngine` |
| Function | `snake_case` | `verify_action` |
| Const | `SCREAMING_SNAKE` | `MAX_RETRIES` |

## Protocol Naming

| Protocol | Enum Variant | Serde Name |
|----------|--------------|------------|
| AgentKern Native | `AgentKern` | `agentkern` |
| Google A2A | `GoogleA2A` | `a2a` |
| Anthropic MCP | `AnthropicMCP` | `mcp` |
| ECMA NLIP | `EcmaNLIP` | `nlip` |

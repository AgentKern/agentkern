# ADR-003: Production-Ready Security Fixes

## Status
Accepted

## Date
2026-01-01

## Context

An Epistemic Debt Audit identified several critical issues in the AgentKern codebase:

1. **GateService Fail-Open Vulnerability**: If the N-API bridge failed to load, the `shouldBlockPrompt()` method returned `false`, allowing all prompts through without security checks.

2. **Missing Configuration Validation**: No fail-fast behavior when required environment variables were missing.

3. **Unsafe Error Handling**: Multiple `unwrap()` calls in Rust code (SSO, integration tests) that could cause panics.

4. **Observability Gaps**: No OpenTelemetry instrumentation or structured logging.

## Decision

### 1. Fail-Closed Security Pattern

Changed `GateService.shouldBlockPrompt()` from:
```typescript
if (!analysis) return false; // Fail-open (INSECURE)
```
To:
```typescript
if (!analysis) return true; // Fail-closed (SECURE)
```

**Rationale**: Security checks that fail open create a false sense of security. If the native bridge is unavailable, blocking all prompts is safer than allowing potentially malicious ones.

### 2. Configuration Validation

Added Joi-based schema validation in `ConfigModule.forRoot()`:
- `DATABASE_URL` required in production
- Sensible defaults for development

### 3. Error Handling

Replaced `unwrap()` with proper error types:
- SSO: `SamlEncodingFailed`, `SystemTimeError`
- Integration tests: `expect()` with descriptive messages

### 4. Observability Stack

- OpenTelemetry SDK for distributed tracing
- Pino for structured JSON logging
- Correlation ID middleware for request tracking

## Consequences

### Positive
- Security bypass no longer possible when bridge unavailable
- Fail-fast startup prevents running with invalid config
- Better debugging with structured logs and traces
- Graceful error handling instead of panics

### Negative
- **Breaking Change**: Environments without compiled bridge will block all prompts
- Slight increase in startup time for validation
- Additional dependencies (OTEL, Pino)

## Alternatives Considered

1. **Keep Fail-Open with Alerts**: Rejected because it still allows attacks during degraded mode.
2. **Startup Failure if Bridge Missing**: Considered too aggressive for development environments.

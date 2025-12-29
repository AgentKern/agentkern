# AgentKern Testing Guide

Complete guide for running security and performance tests.

---

## Quick Start

```bash
# Install dependencies
pnpm install

# Run all tests
pnpm test

# Run with coverage
pnpm test:cov
```

---

## Test Types

### 1. Unit Tests

```bash
# TypeScript (Identity App)
cd apps/identity
pnpm test

# With coverage report
pnpm test:cov

# Watch mode
pnpm test:watch

# Rust (Full workspace)
cargo test --workspace
```

### 2. E2E Tests

```bash
cd apps/identity

# All E2E tests
pnpm test:e2e

# Security-only tests
pnpm test:e2e -- --testPathPattern="security|penetration"

# Specific test file
pnpm test:e2e -- penetration.e2e-spec.ts
```

### 3. Security Tests

```bash
# Comprehensive security E2E suite
pnpm test:e2e -- --testPathPattern="security-comprehensive"

# Penetration tests
pnpm test:e2e -- --testPathPattern="penetration"
```

### 4. Load/Performance Tests

```bash
# Install k6 (if not installed)
# macOS: brew install k6
# Linux: sudo apt install k6

# Basic load test
k6 run tests/performance/gate-load-test.js

# Security load test
k6 run tests/performance/security-load-test.js

# With custom URL
k6 run tests/performance/gate-load-test.js -e GATE_URL=http://localhost:3000
```

---

## Security Scanning

### Pre-Commit Hooks (Recommended)

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually on all files
pre-commit run --all-files
```

### Manual Scans

```bash
# Rust dependency audit
cargo audit

# Rust license check
cargo deny check licenses

# npm audit
cd apps/identity && npm audit --audit-level=high

# Semgrep (if installed)
semgrep scan --config auto .
```

---

## CI/CD Integration

Tests run automatically on:
- **Push to main**: All tests
- **Pull requests**: Unit tests, E2E tests, security scans

### GitHub Actions Jobs

| Job | Trigger | Description |
|-----|---------|-------------|
| `sdk-test` | All | SDK unit tests |
| `gate-test` | All | Rust workspace tests |
| `identity-test` | All | Identity app tests + coverage |
| `security-test` | All | Security E2E tests |
| `rust-coverage` | main only | Tarpaulin coverage |
| `performance-test` | main only | k6 load tests |
| `semgrep-sast` | All | Static analysis |
| `secret-scanning` | All | TruffleHog scan |

---

## Coverage Requirements

| Component | Target | Current |
|-----------|--------|---------|
| Identity App | >80% | Check `coverage/` |
| SDK | >80% | Check reports |
| Rust Core | >70% | Via tarpaulin |

---

## Troubleshooting

### Tests Timeout

```bash
# Increase Jest timeout
pnpm test:e2e -- --testTimeout=30000
```

### Database Connection Issues

```bash
# E2E tests use SQLite in-memory by default
# Check test/jest-e2e.json for configuration
```

### Port Already in Use

```bash
# Kill process on port 3000
kill $(lsof -t -i:3000)
```

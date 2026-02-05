# AgentKern Testing Guide (2026)

This guide covers the verification protocols for the AgentKern Rust runtime and its polyglot SDKs.

---

## 🏗️ Rust Workspace Testing

AgentKern core and pillars are tested using standard Cargo protocols.

### 1. Unit & Integration Tests
```bash
# Run all tests in the workspace (Gate, Synapse, Identity, etc.)
cargo test --workspace

# Run tests for a specific pillar
cargo test -p agentkern-gate

# Run with stdout visible (for debugging)
cargo test -- --nocapture
```

### 2. Security Audits
```bash
# Check for vulnerable dependencies
cargo audit

# Verify license compliance
cargo deny check licenses
```

### 3. Fuzz Testing
We use `cargo-fuzz` for stress-testing parsers (Nexus/Identity).
```bash
cargo fuzz run parse_message
```

---

## 📦 SDK Testing (Polyglot)

### 1. Node.js SDK
```bash
cd sdks/node
pnpm test
```

### 2. Python SDK
```bash
cd sdks/python
pytest
```

---

## 📊 Performance & Load Testing

We use **k6** for high-concurrency load testing of the Unified Server.

```bash
# Install k6: brew install k6 (macOS) or apt install k6 (Linux)

# Run the gate load test
k6 run tests/performance/gate-load-test.js -e GATE_URL=http://localhost:3000
```

---

## 🛡️ Security Verification Suite

All PRs must pass the **Hardened Safety Suite**:

1. **Adversarial Prompts**: Test the `Gate` pillar against known injection patterns.
2. **Identity Revocation**: Verify sub-millisecond propagation of token blacklists.
3. **Atomic Rollback**: Force-fail transactions in `Treasury` to verify 2-phase commit integrity.

---

## 📈 Coverage Requirements

- **Rust Core**: >75% (measured via `cargo-tarpaulin`)
- **SDK Bindings**: >85% (measured via Jest/PyTest)
- **Safety Policies**: 100% path coverage for regulated jurisdictions (EU/US).

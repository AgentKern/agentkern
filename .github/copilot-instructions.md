# AgentKern Copilot Instructions

You are an AI agent working on **AgentKern**, the operating system for autonomous AI agents. This file contains essential knowledge for being immediately productive.

## Architecture Overview

AgentKern solves six fundamental infrastructure problems for multi-agent systems via **Six Pillars**:

```
Gate → Synapse → Arbiter → Treasury → Nexus
(🛡️)    (🧠)     (⚖️)      (💰)      (🔀)
```

### The Six Pillars (All Rust)

| Pillar       | Problem Solved                                     | Key Directory                | Language |
| ------------ | -------------------------------------------------- | ---------------------------- | -------- |
| **Gate**     | Prompt injection defense, policy enforcement       | `packages/pillars/gate/`     | Rust     |
| **Synapse**  | Distributed state, goal drift detection            | `packages/pillars/synapse/`  | Rust     |
| **Arbiter**  | Resource coordination, deadlock prevention         | `packages/pillars/arbiter/`  | Rust     |
| **Treasury** | Atomic payments, carbon tracking                   | `packages/pillars/treasury/` | Rust     |
| **Nexus**    | Multi-vendor protocol translation (MCP, A2A, NLIP) | `packages/pillars/nexus/`    | Rust     |
| **Identity** | Agent authentication & cryptographic proofs        | `packages/pillars/identity/` | Rust     |

### Unified Server Architecture

- **Single Entry Point**: `apps/server/src/main.rs` - Rust Axum gateway for all pillars (no separate microservices)
- **Database**: PostgreSQL (optional; server runs without it for testing)
- **Cache**: Redis (optional)
- **Observability**: OpenTelemetry + structured logging (fail-closed pattern)

**Critical Design Decision (ADR-004)**: **All core pillars are Rust** for zero-copy safety and predictable latency. SDKs (Node.js, Python) provide polyglot access. The Playground (React/Vite) remains TypeScript for UI visualization only.

## Monorepo Structure

```
packages/
  pillars/          # Core implementation (All Rust)
    gate/
    synapse/
    arbiter/
    treasury/
    nexus/
    identity/
  foundation/       # Shared infrastructure (Rust)
    runtime/        # WASM execution for sandboxed policies
    crypto/         # Cryptographic primitives
    pulse/          # Observability & metrics
    governance/     # Compliance & regulations
    parsers/        # Message parsers (IDOC, SWIFT, HL7)
    native-binding/ # N-API bridges to Node.js

apps/
  server/          # Unified Axum gateway (Rust)
  playground/      # React/Vite frontend for mesh visualization (TypeScript only)

sdks/              # Polyglot bindings to Rust core
  core/            # Rust SDK
  node/            # TypeScript/N-API bindings (napi-rs)
  python/          # Python bindings (PyO3/Maturin)

wasm-policies/     # Sandboxed WASM policy modules
  prompt-guard/    # Hot-swappable prompt injection detector
```

## Build & Test Commands

### Rust Workspace (All Pillars + Server)

```bash
# Test all pillar implementations
cargo test --workspace

# Test specific pillar
cargo test -p agentkern-gate
cargo test -p agentkern-synapse
cargo test -p agentkern-arbiter
cargo test -p agentkern-treasury
cargo test -p agentkern-nexus
cargo test -p agentkern-identity

# Build and run the unified server
cargo build -p agentkern-server --release
cargo run -p agentkern-server

# Run with structured logging (debug)
RUST_LOG=debug cargo run -p agentkern-server

# Security audit
cargo audit

# Dependency compliance
cargo deny check licenses

# Fuzzing (Nexus/Identity parsers)
cargo fuzz run parse_message

# Benchmarks (policy evaluation, coordination)
cargo bench --package agentkern-gate --bench policy_eval
cargo bench --package agentkern-arbiter --bench coordination
```

### Node.js SDK (napi-rs)

```bash
cd sdks/node

# Install dependencies
pnpm install

# Build native bindings for current platform
pnpm build --release

# Run tests
pnpm test

# Multi-platform build (requires cross-compiler)
pnpm build --target x86_64-unknown-linux-gnu
pnpm build --target aarch64-apple-darwin
```

### Python SDK (PyO3/Maturin)

```bash
cd sdks/python

# Install in development mode
maturin develop

# Run Python tests
pytest

# Build wheel for distribution
maturin build --release

# Python example
python -c "from agentkern import Agent; agent = Agent.generate('test'); print(agent.id)"
```

### TypeScript/Frontend

```bash
# Install monorepo dependencies
pnpm install

# Build all workspaces (Turbo orchestration)
pnpm build

# Build playground frontend
pnpm build --filter @agentkern/playground

# Format TypeScript/Markdown
pnpm format

# Lint with ESLint
pnpm lint
```

### Chaos Testing (Resilience)

```bash
# Start chaos drill with profiles
./scripts/chaos_drill.sh start light      # 5% failure, 50ms delay
./scripts/chaos_drill.sh start moderate   # 15% failure, 200ms delay
./scripts/chaos_drill.sh start heavy      # 30% failure, 500ms delay

# Run with custom environment variables
export CHAOS_ENABLED=true CHAOS_FAILURE_RATE=0.20 CHAOS_DELAY_MS=300
cargo run -p agentkern-server

# Stop chaos drill
./scripts/chaos_drill.sh stop

# Check chaos status
./scripts/chaos_drill.sh status
```

### Performance Testing

```bash
# Micro-latency benchmarks (Rust)
cargo bench --workspace

# Load test the unified server (requires k6)
k6 run tests/performance/gate-load-test.js -e GATE_URL=http://localhost:3000

# Soak test (long-running stability)
k6 run tests/performance/soak-test.js --duration 10m

```

## Critical Patterns in This Codebase

### 1. **Fail-Closed Security** (ADR-003)

When a security check fails to initialize, the system **blocks by default**, not allows.

**Pattern**: In Gate pillar, if threat analysis unavailable → block all prompts:

```rust
// DON'T: if !analysis { return false; } // Fail-open (dangerous)
// DO:
if !analysis { return true; } // Fail-closed (safe)
```

### 2. **WASM Policy Engine** (Hot-Swap Capability)

Core design choice: policies are sandboxed in WebAssembly for:

- **Sub-microsecond latency**: WASM modules load in microseconds (vs milliseconds for containers)
- **Memory efficiency**: KB instead of MB per policy module
- **Hot-swap**: Replace policies without restarting server
- **Language isolation**: Policy bugs cannot crash core pillars

**Why not containers?** Startup overhead kills microsecond SLA requirements.

```rust
// Gate uses WASM for policy evaluation
let mut engine = WasmPolicyEngine::new()?;
engine.load_policy("spending-limits", wasm_bytes)?;
let result = engine.evaluate(agent_id, action, context)?;

// Hot-swap at runtime
supervisor.send(HotSwapPolicy {
    policy_name: "spending-limits".to_string(),
    wasm_bytes: new_policy_bytes,
}).await?;
```

### 3. **Dependency Ordering**

Pillars depend on Identity first:

- **Identity** (standalone, Ed25519 cryptography)
- **Gate** depends on Identity (verify agent before checking policies)
- **Synapse** depends on Identity (attach state to agent)
- **Arbiter** depends on Identity (verify requester)
- **Treasury** depends on Identity (verify sender/receiver)
- **Nexus** depends on all others (translate protocols while maintaining guarantees)

### 4. **Structured Error Handling**

Never use `.unwrap()` in production code. Use proper error types:

```rust
// BAD: sso.encode().unwrap()
// GOOD:
let encoded = sso.encode()
    .map_err(|e| SamlEncodingFailed(e))?;
```

### 5. **Observability as Default**

All major operations log with OpenTelemetry spans. When adding features:

- Add `#[tracing::instrument]` to public functions
- Use structured fields: `tracing::info!("event", field = value)`
- Emit spans for distributed tracing (Tempo)
- Export Prometheus metrics

### 6. **RAII & Resource Guards**

Rust patterns heavily used in Treasury (2-phase commit) and Arbiter (lock guards). Understand RAII lifecycle:

```rust
// Automatic cleanup when guard drops
let _guard = resource.lock()?;
// ... critical section ...
// automatically released here
```

### 7. **Playground is TypeScript UI Only**

The `apps/playground` React/Vite frontend is **NOT core infrastructure**:

- Pure visualization layer for mesh topology
- Uses native SDK bindings to communicate with Rust server
- **No business logic in TypeScript** — all validation/security in Rust pillars
- Safe to iterate rapidly without affecting core security

## Environment Variables & Configuration

Required for production:

```bash
RUST_ENV=production           # Set to "production" for fail-fast validation
JWT_SECRET=<min32chars>       # Cryptographic key for token signing
DATABASE_URL=postgres://...   # PostgreSQL connection
PORT=3000                     # Server listening port
RUST_LOG=info|warn|debug      # Tracing level
```

## Integration Patterns

### Pattern A: Python SDK Integration (Recommended for Python Developers)

Use PyO3 bindings for native performance:

```python
from agentkern import Agent

# Generate agent with Ed25519 keypair
agent = Agent.generate("my-agent")

# Create liability proof (JWT-based authorization)
proof = agent.create_proof("action:payment:transfer")

# Verify proof
is_valid = Agent.verify_proof(proof)

# Full API reference at sdks/python/README.md
```

**Build Python SDK:**

```bash
cd sdks/python
maturin develop  # For dev
maturin build --release  # For distribution
```

### Pattern B: Node.js SDK Integration (TypeScript Environments)

Use N-API bindings for native bridge:

```typescript
import { Agent } from "@agentkern/sdk";

const agent = Agent.generate("my-agent-id");
const proof = agent.createProof("action-name");
```

**Build Node.js SDK:**

```bash
cd sdks/node
pnpm install
pnpm build --release
```

### Pattern C: HTTP API Direct

POST to `/api/v1/<pillar>/verify` with JWT headers.

### Pattern D: Agent-to-Agent Messaging (A2A)

Use A2A protocol for multi-agent coordination via Nexus pillar.

## Testing Requirements

All PRs must pass:

1. **Unit Tests**: `cargo test --workspace`
2. **Security Audit**: `cargo audit` (no vulnerabilities)
3. **License Compliance**: `cargo deny check licenses`
4. **Hardened Safety Suite** (in CI):
   - Adversarial prompts against Gate pillar
   - Identity revocation sub-millisecond verification
   - Treasury atomic rollback simulation

## Key Files to Understand the "Why"

- `docs/STRUCTURE.md` - Monorepo anatomy
- `docs/core/CONCEPTS.md` - Why each pillar exists
- `docs/core/WORKFLOWS.md` - Agent usage patterns
- `docs/governance/adr/ADR-003-production-ready-security-fixes.md` - Fail-closed security philosophy
- `docs/governance/adr/ADR-004-hybrid-language-strategy.md` - All-Rust strategy (updated from hybrid approach)
- `docs/spec/GATE_DESIGN.md` - WASM Policy Engine rationale (lines 881+)
- `docs/spec/INTEGRATION_GUIDE.md` - How to wire up AgentKern
- `scripts/chaos_drill.sh` - Resilience testing with profiles

## Conventions

- **Crate naming**: lowercase-kebab (e.g., `agentkern-gate`)
- **Environment variables**: UPPER_SNAKE_CASE
- **Rust tests**: Inline `#[cfg(test)]` modules with `#[test]` functions
- **TypeScript**: Use Prettier + ESLint; strict mode required
- **Commits**: Use conventional commits (feat:, fix:, docs:, test:)

## Debugging Tips

**Server won't start?**

- Check `RUST_LOG=debug cargo run -p agentkern-server` for structured logs
- Verify `JWT_SECRET` is set (>= 32 bytes)
- If DATABASE_URL missing in production, server exits with clear error

**Test failures in pillars?**

- Run with `--nocapture`: `cargo test -- --nocapture` to see println! output
- Check dependencies are installed: `cargo tree -p pillar-name`

**N-API binding issues?**

- Verify Rust version: `rustc --version` (1.92.0+)
- Rebuild bridge: `cd sdks/node && pnpm build --release`

## Questions to Always Consider

When modifying code in AgentKern:

1. **Does this respect the Pillar's contract?** (Gate blocks, Synapse distributes state, etc.)
2. **Is error handling fail-closed?** (No unwrap(), proper error types)
3. **Does this maintain agent Identity integrity?** (Can we audit who did this?)
4. **Is observability instrumented?** (Can operators debug this in production?)
5. **Does this follow dependency ordering?** (No circular dependencies between pillars)

---

**Repo**: https://github.com/AgentKern/agentkern  
**Latest Version**: 0.1.0-rc1 (2026-01-15)  
**Tests**: 357+ passing | **Security**: Fail-closed | **Languages**: All Rust

## GitHub Actions Workflow Reference

### CI Workflow (`ci.yml`)

**Stages**:

1. **Rust Workspace Tests** (`rust-test`): Runs `cargo test --workspace` with PostgreSQL, clippy, formatting checks
2. **SDK Tests** (`sdk-test`): Builds Node.js SDK with N-API, runs `pnpm test`
3. **E2E Tests** (`identity-e2e`): Starts agentkern-server, health-checks all 6 pillars via HTTP
4. **Playground Build** (`playground-build`): Builds React/Vite frontend
5. **Docker Build** (`docker-build`): Multi-stage Docker image on main branch

**Key Environment Variables**:

```bash
JWT_SECRET=agentkern-dev-secret-DO-NOT-USE-IN-PRODUCTION (min 32 chars)
AGENTKERN_LICENSE_KEY=PRO-TEST-LICENSE-KEY-1234567890123456789
POLICY_DIR=./policies
PYO3_PYTHON=python3  # For PyO3/Maturin bindings
```

### SDK Build Workflow (`sdk-build.yml`)

**Builds**:

- **Node.js**: Multi-platform (Linux x86_64/ARM64, macOS, Windows) via `pnpm build --target`
- **Python**: Wheels via `maturin build --release` with `PyO3/maturin-action@v1`
- **Publish**: On release tag → npm registry + PyPI

**Common Failures**:

- Python wheels fail if Rust version mismatches Python version in `pyproject.toml`
- N-API bindings require system Perl on Linux (`perl-IPC-Cmd`, `perl-Time-Piece`)
- Node.js multi-platform builds need `cross` or explicit target toolchains

### Security Workflow (`security.yml`)

**Jobs**:

1. **SBOM Generation**: `cargo sbom` (Rust) + `cyclonedx` (TypeScript)
2. **Dependency Audit**: `cargo audit` + `cargo deny check licenses`
3. **SLSA L3 Provenance**: Build provenance attestation
4. **Trivy Container Scan**: Docker image vulnerability scanning

### Coverage Workflow (`coverage.yml`)

Runs `cargo tarpaulin --workspace --out Html` with PostgreSQL, uploads HTML report to artifacts.

### Performance Workflow (`performance.yml`)

**Status**: Partially disabled (k6 tests being restored)

- Enabled: Rust micro-latency benchmarks (`cargo bench`)
- Disabled: k6 load tests (waiting for Rust server k6 scripts)

### Common CI Failures & Fixes

#### 1. **Docker Build Fails: "disk space"**

**Cause**: Multi-stage Rust compile exhausts GitHub runner disk
**Fix** (already in `.github/workflows/ci.yml`):

```yaml
- name: Free Disk Space (Aggressive)
  run: |
    sudo rm -rf /usr/share/dotnet
    sudo rm -rf /usr/local/lib/android
    sudo rm -rf /opt/ghc
    sudo docker image prune --all --force
```

#### 2. **E2E Tests Timeout: "Server not healthy"**

**Cause**: `agentkern-server` takes >30 seconds to start; JWT_SECRET or DB misconfig
**Fix**:

- Increase health-check timeout from 30 to 60 retries
- Verify `JWT_SECRET` is set (≥32 bytes)
- Check PostgreSQL migrations: `sqlx migrate run` in both `arbiter/` and `identity/` directories

#### 3. **Python SDK Build Fails: "SOABI mismatch"**

**Cause**: PyO3/Maturin expects Python version to match OS
**Fix**: Explicitly set `PYO3_PYTHON` environment variable:

```yaml
env:
  PYO3_PYTHON: python3.11
```

#### 4. **Node.js Multi-Platform ARM64 Fails**

**Cause**: Missing `aarch64-linux-gnu-gcc` cross-compiler
**Fix** (already in `sdk-build.yml`):

```yaml
- name: Install cross-linker (Linux ARM64)
  if: matrix.target == 'aarch64-unknown-linux-gnu'
  run: |
    sudo apt-get update
    sudo apt-get install -y gcc-aarch64-linux-gnu
    echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> $GITHUB_ENV
```

#### 5. **Clippy Warnings Block Build: "-D warnings"**

**Cause**: Rust compiler warnings treated as errors in CI
**Fix**: In `Cargo.toml` or GitHub Actions, allow specific warnings:

```bash
cargo clippy --workspace -- -D warnings
# Fix warnings by running:
cargo fix --allow-dirty --allow-staged
```

### Local Testing Before Push

Simulate CI locally:

```bash
# Test everything CI runs
./scripts/run_workspace_tests.sh

# Or manually:
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
pnpm install && pnpm build
cd apps/server && cargo build --release
```

### Debugging CI Failures

**View logs**:

- GitHub: Actions tab → Workflow run → Job logs
- Download artifacts: Build outputs, server logs, coverage reports

**Reproduce locally**:

```bash
# Recreate CI environment
export DATABASE_URL="postgres://postgres:root@localhost:5432/agentkern_test"
export JWT_SECRET="agentkern-dev-secret-DO-NOT-USE-IN-PRODUCTION"
export RUST_LOG=debug

docker-compose up postgres -d
cargo test --workspace
```

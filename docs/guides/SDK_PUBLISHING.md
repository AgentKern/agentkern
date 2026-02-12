# SDK Publishing Guide (2026)

This guide covers publishing the AgentKern polyglot SDKs to npm and PyPI.

## 📦 Prerequisites

### npm (Node.js SDK)
```bash
npm login
# Verify: npm whoami
```

### PyPI (Python SDK)
```bash
pip install twine
# Ensure ~/.pypirc has your API token
```

---

## 🚀 Node.js SDK (`@agentkern/sdk`)

**Path**: `sdks/node`

### 1. Build Native Bindings
The Node.js SDK is compiled from Rust core crates and packaged for Node.js.

```bash
cd sdks/node

# Install dependencies
pnpm install

# Build for current platform
pnpm build

# Build for all platforms (CI only)
pnpm build -- --target x86_64-unknown-linux-gnu
pnpm build -- --target aarch64-unknown-linux-gnu
pnpm build -- --target x86_64-apple-darwin
pnpm build -- --target aarch64-apple-darwin
```

### 2. Verify Locally
```bash
# Test the build
node -e "const { Agent } = require('.'); console.log(Agent.generate('test').id)"
```

### 3. Publish to npm
```bash
# Dry run
npm publish --dry-run

# Publish public access
npm publish --access public
```

---

## 🐍 Python SDK (`agentkern`)

**Path**: `sdks/python`

### 1. Build Wheels
We use `maturin` to build Python wheels from Rust.

```bash
cd sdks/python

# Build for current platform
maturin build --release
```

### 2. Publish to PyPI
```bash
# Build and upload in one step
maturin publish
```

---

## 🤖 CI/CD Automation

SDKs are automatically published via GitHub Actions when a tag starting with `v*` is pushed.

- **Workflow**: `.github/workflows/publish-sdks.yml`
- **Triggers**: `v0.2.0`, `v0.2.1`, etc.

### Version Management
Ensure versions match in:
1. `sdks/node/package.json`
2. `sdks/python/pyproject.toml`
3. `sdks/core/Cargo.toml` (if core changed)

# SDK Publishing Guide

This guide covers publishing the AgentKern SDKs to npm and PyPI.

## Prerequisites

### npm (Node.js SDK)
```bash
npm login
# Verify: npm whoami
```

### PyPI (Python SDK)
```bash
pip install twine
# Create ~/.pypirc with API token
```

---

## Node.js SDK (@agentkern/sdk)

### 1. Build Native Bindings

```bash
cd packages/sdk-node

# Install dependencies
pnpm install

# Build for current platform
pnpm build

# Build for all platforms (requires cross-compilation)
pnpm build -- --target x86_64-unknown-linux-gnu
pnpm build -- --target aarch64-unknown-linux-gnu
pnpm build -- --target x86_64-apple-darwin
pnpm build -- --target aarch64-apple-darwin
pnpm build -- --target x86_64-pc-windows-msvc
```

### 2. Test Locally

```bash
# Create test project
mkdir test-sdk && cd test-sdk
npm init -y

# Install local package
npm install ../packages/sdk-node

# Test
node -e "
const { Agent } = require('@agentkern/sdk');
const agent = Agent.generate('test');
console.log('Agent ID:', agent.id);
"
```

### 3. Publish to npm

```bash
cd packages/sdk-node

# Dry run
npm publish --dry-run

# Publish
npm publish --access public
```

### 4. Verify

```bash
npm view @agentkern/sdk
npm install @agentkern/sdk
```

---

## Python SDK (agentkern)

### 1. Build Wheels

```bash
cd sdks/python

# Install maturin
pip install maturin

# Build for current platform
maturin build --release

# Build for multiple platforms (requires Docker)
docker run --rm -v $(pwd):/io \
  ghcr.io/pyo3/maturin build --release --manylinux 2014

# Wheels will be in target/wheels/
```

### 2. Test Locally

```bash
# Install in development mode
maturin develop

# Test
python3 -c "
from agentkern import Agent
agent = Agent.generate('test')
print(f'Agent ID: {agent.id}')
"
```

### 3. Publish to PyPI

```bash
cd sdks/python

# Build source distribution
maturin build --release --sdist

# Upload to PyPI
twine upload target/wheels/*

# Or use maturin directly
maturin publish
```

### 4. Verify

```bash
pip install agentkern
python3 -c "from agentkern import Agent; print(Agent.generate('test').id)"
```

---

## GitHub Releases

### 1. Create Release

```bash
gh release create v0.2.0 \
  --title "v0.2.0: SDK Infrastructure + Security Hardening" \
  --notes-file CHANGELOG.md \
  target/wheels/*.whl \
  packages/sdk-node/*.node
```

### 2. Attach Binaries

Upload pre-built binaries for common platforms:
- `agentkern-sdk-darwin-x64.node`
- `agentkern-sdk-darwin-arm64.node`
- `agentkern-sdk-linux-x64.node`
- `agentkern-sdk-linux-arm64.node`
- `agentkern-0.2.0-cp310-abi3-manylinux_2_17_x86_64.whl`
- `agentkern-0.2.0-cp310-abi3-macosx_11_0_arm64.whl`

---

## CI/CD Automation

### GitHub Actions Workflow

```yaml
# .github/workflows/publish-sdks.yml
name: Publish SDKs

on:
  push:
    tags:
      - 'v*'

jobs:
  publish-npm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: cd packages/sdk-node && pnpm install && pnpm build
      - run: npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  publish-pypi:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          command: publish
          args: --manifest-path sdks/python/Cargo.toml
        env:
          MATURIN_PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
```

---

## Version Management

Update version in all places:
1. `packages/sdk-core/Cargo.toml`
2. `packages/sdk-node/Cargo.toml`
3. `packages/sdk-node/package.json`
4. `sdks/python/Cargo.toml`
5. `sdks/python/pyproject.toml`
6. `CHANGELOG.md`

---

## Troubleshooting

### npm: 404 Not Found
- Ensure package name is available
- Check org scope (@agentkern)
- Verify npm login

### PyPI: Invalid Distribution
- Ensure wheel is built for correct platform
- Check Python version compatibility (>=3.10)
- Verify manylinux compliance

### Native Module Won't Load
- Check ABI compatibility
- Verify platform target matches
- Rebuild for specific Node.js version

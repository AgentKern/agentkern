# Open Source Setup Guide

This guide explains how AgentKern is structured as an open source project and how to work with it.

---

## 📋 License Structure

AgentKern uses an **Open Core** model:

### Open Source (Apache 2.0)
- ✅ **All packages in `packages/`** - Core pillars and foundation
- ✅ **All apps in `apps/`** - Server and playground
- ✅ **All SDKs in `sdks/`** - Client libraries
- ✅ **All documentation in `docs/`**

### Enterprise (Proprietary)
- ❌ **`ee/` directory** - Enterprise features (not included in OSS)
  - Energy Pillar extensions
  - Advanced Treasury features
  - Sovereign Memory encryption
  - Multi-cloud mesh sync

---

## 🏗️ Project Structure

```
agentkern/
├── apps/
├── packages/
├── sdks/
├── docs/
├── tests/
└── ee/                  # ❌ (Not present in OSS - Overlay target)
```

---

## 🔌 Enterprise Integration

The Enterprise Edition is designed as an overlay. When EE assets are available, they can be integrated into the build process to enable advanced features.

### How It Works

1. **Sync Overlay**: Pull the private `agentkern-ee` repository into local `ee/`.
2. **Enable EE Members**: Add EE crates into workspace members.
3. **License Check**: Server checks `AGENTKERN_LICENSE_KEY`.

### Integration Path

For enterprise customers:
1. Obtain enterprise license
2. Sync enterprise overlay:
   ```bash
   ./scripts/pull-ee.sh
   ```
3. Enable EE workspace members:
   ```bash
   ./ee/scripts/init-workspace.sh
   ```
4. Set `AGENTKERN_LICENSE_KEY` environment variable
5. Build and run:
   ```bash
   cargo build --workspace
   ```
6. Enterprise endpoints become active

---

## 📦 Building OSS Only

### Standard Build (OSS)

```bash
# Build everything (OSS only)
cargo build --workspace

# Build server
cargo build --bin agentkern-server --release

# Run tests
cargo test --workspace
```

### What Gets Built

- ✅ All six pillars (Identity, Gate, Synapse, Arbiter, Nexus, Treasury)
- ✅ Unified server (`agentkern-server`)
- ✅ SDKs (Rust, Node.js, Python)
- ✅ Playground frontend

### What Doesn't Get Built

- ❌ Enterprise modules (not in repository)
- ❌ Enterprise overlay endpoints return "quarantined" status
- ❌ Treasury HTTP routes in the unified OSS server are quarantined by default

If you previously enabled EE members and want to return to OSS-only workspace mode:

```bash
./ee/scripts/reset-workspace.sh
```

---

## 🚀 Running OSS Server

```bash
# Start server
cargo run --bin agentkern-server

# With environment variables
DATABASE_URL=postgres://... \
REDIS_URL=redis://... \
cargo run --bin agentkern-server
```

### Available Endpoints (OSS)

- `/health` - Health check
- `/api/v1/identity/*` - Identity pillar
- `/api/v1/gate/*` - Gate pillar
- `/api/v1/synapse/*` - Synapse pillar
- `/api/v1/arbiter/*` - Arbiter pillar
- `/api/v1/nexus/*` - Nexus pillar
- `/api/v1/treasury/*` - Quarantined in default OSS server mode

See [OSS_CAPABILITY_MATRIX.md](OSS_CAPABILITY_MATRIX.md) for canonical route and pillar availability.

---

## 📝 Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Code standards
- Testing requirements
- Pull request process
- Architecture decisions

---

## 🔒 Security

### OSS Security

- All OSS code is auditable
- Security vulnerabilities: See [SECURITY.md](../SECURITY.md)
- Responsible disclosure: security@agentkern.io

### Enterprise Security

- Enterprise modules are separately licensed
- Additional security features in enterprise edition
- Contact sales@agentkern.io for enterprise security details

---

## 📚 Documentation

### OSS Documentation

- **Architecture**: `docs/ARCHITECTURE_FINAL.md`
- **Quick Start**: `docs/QUICKSTART.md`
- **API Reference**: `docs/reference/`
- **Guides**: `docs/guides/`
- **Specifications**: `docs/spec/`

### Enterprise Documentation

- Enterprise features documented separately
- Available to enterprise license holders
- Contact support@agentkern.io for access

---

## 🎯 Roadmap

### OSS Roadmap

- Core pillar improvements
- Protocol support (A2A, MCP, ANP, NLIP)
- SDK enhancements
- Performance optimizations
- Community-driven features

### Enterprise Roadmap

- Advanced compliance features
- Multi-cloud mesh sync
- Enterprise integrations (SAP, SWIFT, etc.)
- Managed services

---

## 💬 Community

- **GitHub Discussions**: For questions and discussions
- **GitHub Issues**: For bugs and feature requests
- **Contributing**: See [CONTRIBUTING.md](../CONTRIBUTING.md)

---

## 📄 License

- **OSS Components**: Apache 2.0 (see [LICENSE](../LICENSE))
- **Enterprise Components**: Commercial License (see `ee/LICENSE-ENTERPRISE.md` if available)

---

**AgentKern is open source. The core is free forever. Enterprise features are available for production deployments.**

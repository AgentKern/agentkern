# AgentKern Developer Setup & Configuration

This guide covers the local development environment setup and environment-specific configurations for AgentKern.

## 📋 System Requirements

To build and run AgentKern locally, you need:

- **Rust**: 1.92.0 or later (Stable)
- **Node.js**: 20.x or later
- **pnpm**: 9.x or later
- **Docker & Docker Compose**: For database and cache services
- **Protobuf Compiler (`protoc`)**: For gRPC and message serialization
- **OpenSSL Development Headers**: For cryptographic operations

### 🐧 Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y protobuf-compiler libssl-dev pkg-config build-essential clang cmake
```

### 🍎 macOS
```bash
brew install protobuf openssl pkg-config
```

---

## 🛠️ Local Environment Setup

### 1. Clone & Install
```bash
git clone https://github.com/agentkern/agentkern.git
cd agentkern
pnpm install
```

### 2. Start Infrastructure
```bash
docker-compose up -d postgres redis
```

### 3. Build & Run
```bash
cargo run -p agentkern-server
```

---

## ⚙️ Configuration (Environment Variables)

AgentKern uses environment variables for configuration. Copy `.env.example` to `.env` to start.

| Variable | Description | Production Requirement |
|----------|-------------|------------------------|
| `RUST_ENV` | Environment name | Set to `production` |
| `JWT_SECRET` | Secret key for JWT signing | Minimum 32 bytes |
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `PORT` | Server listening port | Default 3000 |
| `RUST_LOG` | Logging level | Set to `info` or `warn` |

### Example `.env`
```ini
RUST_ENV=development
JWT_SECRET=dev_secret_key_at_least_32_chars_long
DATABASE_URL=postgres://localhost/agentkern
PORT=3000
RUST_LOG=debug
```

---

## 💻 IDE Configuration (VS Code)

Install the following extensions:
- **rust-analyzer**: The essential Rust language server
- **Even Better TOML**: For `Cargo.toml` highlighting
- **ESLint & Prettier**: For TypeScript formatting

Add the following to `.vscode/settings.json`:
```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.features": "all"
}
```

---

## 🚀 Running Sub-projects

- **Playground**: `cd apps/playground && npm run dev`
- **Tests**: `cargo test -p agentkern-gate`

## ⚠️ Common Issues

- **"JWT_SECRET too short"**: Ensure your secret is >= 32 characters in production.
- **"Protoc not found"**: Verify `protoc` is in your system PATH.

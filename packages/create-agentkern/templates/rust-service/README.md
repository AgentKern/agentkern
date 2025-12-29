# {{PROJECT_NAME}}

An AgentKern service with Zero-Trust security defaults.

## Features

- **mTLS** - Mutual TLS authentication built-in
- **Gate Integration** - Policy engine and observability
- **OpenTelemetry** - Distributed tracing ready

## Quick Start

```bash
cargo build
cargo run
```

## Configuration

Environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `MTLS_CERT_PATH` | Path to TLS certificate | `./certs/server.crt` |
| `MTLS_KEY_PATH` | Path to TLS private key | `./certs/server.key` |
| `GATE_POLICY_PATH` | Path to policy rules | `./policies/` |

## License

Apache-2.0

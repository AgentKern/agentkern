# AgentKern Python SDK

Native Ed25519 cryptography and Liability Proof creation for Python.

## Installation

```bash
pip install agentkern
```

Or build from source:

```bash
cd sdks/python
maturin develop
```

## Quick Start

```python
from agentkern import Agent

# Generate a new agent with Ed25519 keypair
agent = Agent.generate("my-agent")
print(f"Agent ID: {agent.id}")  # did:key:z...

# Create a liability proof
proof = agent.create_proof("payment:transfer:100")
print(f"JWT: {proof.jwt}")

# Verify the proof
is_valid = Agent.verify_proof(proof)
print(f"Valid: {is_valid}")  # True
```

## Features

- **Ed25519 Cryptography**: Native Rust performance via PyO3
- **Liability Proofs**: JWT-based authorization with hardware-bound credentials
- **A2A Protocol**: Agent-to-agent message encoding
- **Type Hints**: Full `.pyi` stub files for IDE autocomplete

## API Reference

### Agent

```python
# Generate
agent = Agent.generate("name")
agent = Agent.generate_with_config("name", proof_expiry_seconds=600)

# Restore from seed
agent = Agent.from_seed("name", seed_base64)

# Properties
agent.id          # DID (did:key:z...)
agent.name        # Agent name
agent.public_key  # Base64url public key
agent.seed        # Base64url seed (SENSITIVE)

# Sign
signature = agent.sign(b"data")
signature = agent.sign_message("message")

# Proofs
proof = agent.create_proof("action")
is_valid = Agent.verify_proof(proof)
```

### LiabilityProof

```python
proof.issuer      # Issuer DID
proof.subject     # Agent DID
proof.action      # Authorized action
proof.expires_at  # Unix timestamp
proof.jwt         # Full JWT string
proof.is_expired()  # Check expiration
```

## License

MIT

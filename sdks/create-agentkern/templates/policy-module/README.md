# {{PROJECT_NAME}}

A WASM Policy Module for [AgentKern Gate](https://github.com/AgentKern/agentkern).

## Features

- 🔒 **Custom Policy Rules**: Enforce custom security policies
- ⚡ **WASM Performance**: Near-native execution speed
- 🔌 **Hot-Reload**: Deploy without restarting Gate

## Building

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build release WASM
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/{{PROJECT_NAME_SNAKE}}.wasm
```

## Deploying

```bash
# Copy to Gate's policy directory
cp target/wasm32-unknown-unknown/release/{{PROJECT_NAME_SNAKE}}.wasm /path/to/gate/policies/

# Or upload via API
curl -X POST http://localhost:8080/policies \
  -F "policy=@target/wasm32-unknown-unknown/release/{{PROJECT_NAME_SNAKE}}.wasm"
```

## Policy Rules

Edit `src/lib.rs` to customize your policy rules:

```rust
fn evaluate_action(ctx: &ActionContext) -> PolicyDecision {
    // Add your rules here
}
```

## Testing

```bash
cargo test
```

## Learn More

- [Gate Policy Documentation](https://github.com/AgentKern/agentkern/docs/policies)
- [WASM Component Model](https://component-model.bytecodealliance.org/)

# Dependency Audit Report

## Summary
cargo-machete found unused dependencies across the workspace.

## Priority: Core Pillars

### agentkern-gate
```
agentkern-multitenancy  # Review: may be used in feature-gated code
ml-dsa                  # Review: used in `pqc` feature
```

### agentkern-synapse
```
anyhow       # Error handling (may be transitive)
arrow        # Data format (may be feature-gated)
crdts-lib    # CRDTs (review usage)
petgraph     # Graph structures (review usage)
polars       # DataFrame (may be feature-gated)
tower        # HTTP layer (may be unused)
```

### agentkern-arbiter
```
anyhow        # Error handling
async-trait   # Async traits
openraft      # Raft consensus (feature-gated)
tokio-uring   # io_uring (optional)
tower         # HTTP layer
```

### agentkern-treasury
```
redis         # Cache (may be optional feature)
serde_json    # JSON (likely used)
tower         # HTTP layer
tower-http    # HTTP middleware
```

### agentkern-nexus
```
anyhow  # Error handling
axum    # HTTP (may be in server binary)
base64  # Encoding
prost   # Protobuf
tonic   # gRPC
tower   # HTTP layer
```

## Priority: Foundation

### agentkern-parsers
```
regex        # Pattern matching
serde_json   # JSON parsing
thiserror    # Error handling
```

### agentkern-governance
```
regex              # Pattern matching
rust_decimal       # Decimal math
rust_decimal_macros # Decimal macros
```

## Enterprise Edition (ee/)
- 18 crates with unused dependencies
- Most are scaffolded/placeholder implementations
- Lower priority for cleanup

## Recommendations
1. Many "unused" deps may be feature-gated or used in optional code
2. Run with `--with-metadata` for more accurate analysis
3. Focus on core pillars first before ee/ cleanup
4. Some deps (tokio, serde_json) are likely false positives

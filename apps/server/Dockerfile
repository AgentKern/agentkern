# Build stage
FROM rust:1.92-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
# split into minimal layers for better caching
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    libssl-dev \
    pkg-config \
    build-essential \
    clang \
    cmake \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifest files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY packages/foundation/crypto/Cargo.toml packages/foundation/crypto/Cargo.toml
COPY packages/foundation/gate-policy/Cargo.toml packages/foundation/gate-policy/Cargo.toml
COPY packages/foundation/governance/Cargo.toml packages/foundation/governance/Cargo.toml
COPY packages/foundation/parsers/Cargo.toml packages/foundation/parsers/Cargo.toml
COPY packages/foundation/pulse/Cargo.toml packages/foundation/pulse/Cargo.toml

# Copy pillar manifests
COPY packages/pillars/arbiter/Cargo.toml packages/pillars/arbiter/Cargo.toml
COPY packages/pillars/gate/Cargo.toml packages/pillars/gate/Cargo.toml
COPY packages/pillars/nexus/Cargo.toml packages/pillars/nexus/Cargo.toml
COPY packages/pillars/synapse/Cargo.toml packages/pillars/synapse/Cargo.toml

# Copy application manifest
COPY apps/server/Cargo.toml apps/server/Cargo.toml
COPY ee/multitenancy/Cargo.toml ee/multitenancy/Cargo.toml
COPY ee/core/Cargo.toml ee/core/Cargo.toml
COPY ee/connectors/Cargo.toml ee/connectors/Cargo.toml

# Copy full source
COPY packages ./packages
COPY ee ./ee
COPY sdks ./sdks
COPY apps/server ./apps/server

# Build release binary
RUN cargo build -p agentkern-server --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/agentkern-server /usr/local/bin/

# Expose default port
EXPOSE 3000

# Run the server
CMD ["agentkern-server"]

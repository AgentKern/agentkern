# Base stage
FROM rust:1.92-slim-bookworm AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

# Planner stage
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Install system dependencies needed for compiling dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    libssl-dev \
    pkg-config \
    build-essential \
    clang \
    cmake \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies - this is the cached layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release -p agentkern-server

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security (prevent privilege escalation)
RUN useradd -m -u 1000 agentkern && \
    mkdir -p /app && \
    chown -R agentkern:agentkern /app

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/agentkern-server /usr/local/bin/

# Switch to non-root user
USER agentkern

# Expose default port
EXPOSE 3000

# Health check for orchestration (Docker, Kubernetes, etc.)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the server
CMD ["agentkern-server"]

# Build stage
FROM rust:1.84-bookworm AS builder

WORKDIR /app

# Copy workspace files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY packages ./packages
COPY apps/server ./apps/server

# Build release binary
RUN cargo build -p agentkern-server --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/agentkern-server /usr/local/bin/

# Expose default port
EXPOSE 3000

# Run the server
CMD ["agentkern-server"]

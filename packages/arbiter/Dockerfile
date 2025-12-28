# Build stage using workspace context
# Must be built from repository root: docker build -f packages/arbiter/Dockerfile .
FROM rust:1.83-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconf openssl-dev

# Copy workspace files first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Copy all workspace crates (needed for dependency resolution)
COPY packages packages/
COPY ee ee/
COPY crates crates/

# Build release binary
RUN cargo build --release --package agentkern-arbiter --bin arbiter-server

# Runtime stage - minimal image
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/arbiter-server /app/arbiter-server

# Set environment
ENV PORT=3003
ENV RUST_LOG=info

EXPOSE 3003

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3003/health || exit 1

CMD ["/app/arbiter-server"]

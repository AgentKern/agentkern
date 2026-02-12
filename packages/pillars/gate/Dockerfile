# Build stage using workspace context
# Must be built from repository root: docker build -f packages/gate/Dockerfile .
FROM rust:1.83-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconf openssl-dev

# Copy workspace files first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Copy all workspace crates (needed for dependency resolution)
COPY packages packages/
COPY ee ee/

# Build release binary
RUN cargo build --release --package agentkern-gate --bin gate-server

# Runtime stage - minimal image
FROM alpine:3.19

RUN apk add --no-cache ca-certificates \
    && addgroup -S agentkern \
    && adduser -S -u 1000 -G agentkern agentkern

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/gate-server /app/gate-server
RUN chown -R agentkern:agentkern /app

# Set environment
ENV PORT=3001
ENV RUST_LOG=info

EXPOSE 3001

USER agentkern

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

CMD ["/app/gate-server"]

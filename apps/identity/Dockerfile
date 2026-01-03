# ========================
# Rust Build Stage (for N-API Bridge)
# ========================
FROM rust:1.75-alpine AS rust-builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig

# Copy Rust workspace files
COPY Cargo.toml Cargo.lock ./
COPY packages/foundation/bridge/Cargo.toml ./packages/foundation/bridge/
COPY packages/pillars ./packages/pillars/
COPY packages/foundation/bridge/src ./packages/foundation/bridge/src/

# Build N-API bridge
WORKDIR /app/packages/foundation/bridge
RUN cargo build --release

# ========================
# Node.js Build Stage
# ========================
FROM node:20-alpine AS builder

WORKDIR /app

# Install all dependencies for build
RUN npm install -g pnpm

COPY package*.json pnpm-lock.yaml ./

RUN pnpm install --frozen-lockfile

# Copy source
COPY . .

# Copy built bridge from Rust stage
COPY --from=rust-builder /app/packages/foundation/bridge/index.node ./packages/foundation/bridge/

# Verify bridge exists before building
RUN test -f packages/foundation/bridge/index.node || \
    (echo "❌ CRITICAL: N-API bridge not found" && exit 1)

# Build Identity app
WORKDIR /app/apps/identity
RUN pnpm run build

RUN pnpm prune --prod

# ========================
# Production Stage
# ========================
FROM node:20-alpine AS production

WORKDIR /app

# Security: Run as non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S agentproof -u 1001

# Copy built application and prod deps
COPY --from=builder /app/apps/identity/dist ./dist
COPY --from=builder /app/apps/identity/node_modules ./node_modules
COPY --from=builder /app/apps/identity/package*.json ./

# Copy N-API bridge (critical for production)
COPY --from=builder /app/packages/foundation/bridge/index.node ./packages/foundation/bridge/

# Verify bridge is accessible from dist location
RUN test -f packages/foundation/bridge/index.node || \
    (echo "❌ CRITICAL: Bridge not found in production image" && exit 1)

# Set environment
ENV NODE_ENV=production
ENV PORT=3000

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Run as non-root
USER agentproof

# Start server
CMD ["node", "dist/main.js"]

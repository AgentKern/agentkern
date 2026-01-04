#!/bin/bash
# Build script for AgentKern Node.js SDK
# Builds native bindings for multiple platforms

set -e

echo "🔨 Building @agentkern/sdk (Node.js)"

cd packages/sdk-node

# Install dependencies
echo "📦 Installing dependencies..."
pnpm install

# Build for current platform
echo "🏗️  Building for current platform..."
pnpm build --release

# Check if cross-compilation is available
if command -v cross &> /dev/null; then
    echo "🌍 Building for additional platforms..."
    
    # Linux x86_64
    echo "  - Linux x86_64"
    cargo build --release --target x86_64-unknown-linux-gnu
    
    # Linux ARM64
    echo "  - Linux ARM64"
    cargo build --release --target aarch64-unknown-linux-gnu
    
    # macOS Intel
    echo "  - macOS x86_64"
    cargo build --release --target x86_64-apple-darwin
    
    # macOS Apple Silicon
    echo "  - macOS ARM64"
    cargo build --release --target aarch64-apple-darwin
    
    echo "✅ Multi-platform build complete"
else
    echo "⚠️  cross tool not found, skipping multi-platform builds"
    echo "   Install with: cargo install cross"
fi

echo ""
echo "✨ Build complete!"
echo "📦 Artifacts:"
ls -lh *.node 2>/dev/null || echo "  *.node files in target/release/"
echo ""
echo "To publish:"
echo "  npm publish --access public"

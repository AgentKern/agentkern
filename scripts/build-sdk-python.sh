#!/bin/bash
# Build script for AgentKern Python SDK
# Builds wheels for multiple Python versions and platforms

set -e

echo "🔨 Building agentkern (Python)"

cd sdks/python

# Install maturin if not available
if ! command -v maturin &> /dev/null; then
    echo "📦 Installing maturin..."
    pip install maturin
fi

# Build for current platform
echo "🏗️  Building for current platform..."
maturin build --release

# Check if Docker is available for manylinux builds
if command -v docker &> /dev/null; then
    echo "🐳 Building manylinux wheels..."
    
    docker run --rm -v $(pwd):/io \
        ghcr.io/pyo3/maturin build --release \
        --manylinux 2014 \
        --interpreter python3.10 python3.11 python3.12 python3.13
    
    echo "✅ Manylinux wheels built"
else
    echo "⚠️  Docker not found, skipping manylinux builds"
fi

echo ""
echo "✨ Build complete!"
echo "📦 Artifacts in target/wheels/:"
ls -lh target/wheels/
echo ""
echo "To publish:"
echo "  maturin publish"
echo "  # or"
echo "  twine upload target/wheels/*"

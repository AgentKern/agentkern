#!/bin/bash
# AgentKern Workspace Test Helper
# This script attempts to run the full workspace test suite, handling optional dependencies.

echo "🚀 Starting AgentKern Workspace Test Suite..."

# 1. Check for database
if [ -z "$DATABASE_URL" ]; then
    echo "⚠️  DATABASE_URL not set. Some integration tests (Arbiter, identity-db) will be skipped or fail."
    echo "💡 Set DATABASE_URL to a valid Postgres connection string to enable full testing."
    SKIP_DB_TESTS=true
fi

# 2. Run core foundation tests
echo "📦 Testing foundation packages..."
cargo test -p agentkern-crypto -p agentkern-edge -p agentkern-governance -p agentkern-pulse -p agentkern-runtime

# 3. Run pillar tests
echo "🏛️ Testing pillars..."
if [ "$SKIP_DB_TESTS" = true ]; then
    # Filter out tests that require DB
    cargo test -p agentkern-gate -p agentkern-synapse -p agentkern-nexus -p agentkern-treasury
    cargo test -p agentkern-identity --lib # Run identity unit tests only
else
    cargo test --workspace
fi

# 4. Check results
if [ $? -eq 0 ]; then
    echo "✅ Workspace tests passed!"
else
    echo "❌ Some tests failed. Check the output above."
    exit 1
fi

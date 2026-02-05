#!/bin/bash
set -e

# 1. Setup environment
export AGENTKERN_LICENSE_KEY="test_license"
export AGENTKERN_LOCAL_KEK=$(openssl rand -base64 32)

echo "🛡️ Sovereign Memory Verification"
echo "==============================="
echo "Local KEK generated: $AGENTKERN_LOCAL_KEK"

# 2. Run unit tests (un-ignoring only the local encryption ones)
echo -e "\nRunning encryption unit tests..."
cargo test -p agentkern-sovereign-memory-ee --lib encryption::tests -- --include-ignored

# 3. Verify Server Integration (if server is reachable)
# Note: This is an optional step if the server is already running
# echo -e "\nTesting Server API..."
# ENCRYPT_RESP=$(curl -s -X POST http://localhost:8080/api/v1/ee/memory/encrypt -H "Content-Type: application/json" -d '{"plaintext": "My agent secret thoughts"}')
# echo "Encrypt Resp: $ENCRYPT_RESP"

#!/bin/bash
# Smoke Test Suite for AgentKern
# 
# Run after deployment to verify critical endpoints
# Usage: ./scripts/smoke-test.sh [base_url]

set -e

BASE_URL="${1:-http://localhost:3000}"
PASSED=0
FAILED=0
TOTAL=0

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔬 AgentKern Smoke Tests"
echo "========================"
echo "Target: $BASE_URL"
echo ""

# Helper function for tests
test_endpoint() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local expected_status="$4"
    local body="${5:-}"
    
    TOTAL=$((TOTAL + 1))
    
    if [ -n "$body" ]; then
        response=$(curl -s -o /dev/null -w "%{http_code}" \
            -X "$method" \
            -H "Content-Type: application/json" \
            -d "$body" \
            "$BASE_URL$endpoint" 2>/dev/null || echo "000")
    else
        response=$(curl -s -o /dev/null -w "%{http_code}" \
            -X "$method" \
            "$BASE_URL$endpoint" 2>/dev/null || echo "000")
    fi
    
    if [ "$response" = "$expected_status" ]; then
        echo -e "${GREEN}✓${NC} $name (HTTP $response)"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗${NC} $name - Expected $expected_status, got $response"
        FAILED=$((FAILED + 1))
    fi
}

# Helper for checking response body contains string
test_contains() {
    local name="$1"
    local endpoint="$2"
    local expected_string="$3"
    
    TOTAL=$((TOTAL + 1))

    response=$(curl -s "$BASE_URL$endpoint" 2>/dev/null || echo "")

    if echo "$response" | grep -q "$expected_string"; then
        echo -e "${GREEN}✓${NC} $name"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗${NC} $name - Response doesn't contain '$expected_string'"
        FAILED=$((FAILED + 1))
    fi
}

echo "📍 Health Checks"
echo "----------------"
test_endpoint "Health endpoint" "GET" "/health" "200"
test_contains "Health returns status" "/health" "ok"

echo ""
echo "🔐 Security Endpoints"
echo "---------------------"
test_endpoint "Login rejects missing credentials" "POST" "/api/v1/auth/login" "400" '{}'
test_endpoint "Protected endpoint rejects no auth" "GET" "/api/v1/identity/agents" "401"

echo ""
echo "🔍 Pillar Endpoints"
echo "-------------------"
# Health endpoints should be public and return 200
test_endpoint "Identity health" "GET" "/api/v1/identity/health" "200"
test_endpoint "Gate endpoint exists" "GET" "/api/v1/gate/health" "200"
test_endpoint "Arbiter endpoint exists" "GET" "/api/v1/arbiter/health" "200"
test_endpoint "Nexus endpoint exists" "GET" "/api/v1/nexus/health" "200"
test_endpoint "Synapse endpoint exists" "GET" "/api/v1/synapse/health" "200"

echo ""
echo "========================"
echo "Results: $PASSED/$TOTAL passed"

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}$FAILED tests failed${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi

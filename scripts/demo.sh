#!/bin/bash
# AgentKern Demo Wrapper
set -e

# ANSI Colors
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BOLD}${CYAN}Preparing AgentKern Ground Truth Demo...${NC}"

# Check for database
if [ -z "$DATABASE_URL" ]; then
    echo -e "⚠️  DATABASE_URL not set. Running in stateless mode (stateless demo)."
fi

# Build and run
cargo run -p agentkern-server --bin demo

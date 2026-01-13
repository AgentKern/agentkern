#!/bin/bash
set -e

# Start server
echo "Starting gate-server..."
cargo run --quiet -p agentkern-gate --bin gate-server > server_load.log 2>&1 &
SERVER_PID=$!
echo "Started server with PID $SERVER_PID"

# Wait for health check (max 60s)
echo "Waiting for server health check..."
attempt=0
while [ $attempt -le 60 ]; do
    if curl -s http://localhost:3001/health | grep "healthy" > /dev/null; then
        echo "Server is healthy!"
        ready=true
        break
    fi
    sleep 1
    attempt=$((attempt+1))
done

if [ "$ready" != "true" ]; then
    echo "Server failed to become ready."
    cat server_load.log
    kill $SERVER_PID
    exit 1
fi

# Run k6
echo "Running k6 performance test..."
k6 run tests/performance/gate-load-test.js > k6_results.txt 2>&1

# Display results
echo "=== K6 RESULTS ==="
cat k6_results.txt

# Cleanup
echo "Stopping server..."
kill $SERVER_PID
echo "Done."

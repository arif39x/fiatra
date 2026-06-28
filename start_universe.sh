#!/bin/bash
set -e

cleanup() {
    echo "Shutting down the universe..."
    kill $PID_MATH $PID_SERVER 2>/dev/null
    wait $PID_MATH $PID_SERVER 2>/dev/null
    echo "Uclid Offline."
}
trap cleanup EXIT

echo "Starting Uclid Microservices..."

# Kill leftover processes on our ports
for port in 8080 8081; do
    pid=$(ss -tlnp "sport = :$port" 2>/dev/null | grep -oP 'pid=\K[0-9]+' | head -1)
    if [ -n "$pid" ]; then
        kill "$pid" 2>/dev/null && echo "Killed stale process $pid on port $port"
    fi
done

echo "Booting Compiler (Python)..."
compiler/venv/bin/uvicorn api:app --app-dir compiler --port 8081 &
PID_MATH=$!

sleep 1

echo "Booting Server (Go)..."
(cd server && go run .) &
PID_SERVER=$!

sleep 2

echo "Booting Client (Rust)..."
(cd client && cargo run --release)

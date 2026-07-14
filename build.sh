#!/bin/bash
set -e

cleanup() {
    echo "Shutting down initial..."
    kill $PID_MATH 2>/dev/null
    wait $PID_MATH 2>/dev/null
    echo "initial Offline."
}
trap cleanup EXIT

echo "Starting initial..."

for port in 8081; do
    pid=$(ss -tlnp "sport = :$port" 2>/dev/null | grep -oP 'pid=\K[0-9]+' | head -1)
    if [ -n "$pid" ]; then
        kill "$pid" 2>/dev/null && echo "Killed stale process $pid on port $port"
    fi
done

echo "Booting Compiler (Python)..."
compiler/venv/bin/python -m uvicorn api:app --app-dir compiler --port 8081 &
PID_MATH=$!

sleep 1

echo "Booting Client (Rust)..."
(cd client && cargo run --release)

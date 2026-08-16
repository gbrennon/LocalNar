#!/usr/bin/env bash
# Single-task pi runner — fresh context every invocation.
# Usage: ./pi-task.sh "prompt here"
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODEL="${PI_MODEL:-Qwen3-8B-Q4_K_M}"
PORT="${PORT:-8080}"
CTX_SIZE="${CTX_SIZE:-32768}"
PROMPT="${1:-}"

usage() {
  echo "Usage: pi-task.sh <prompt>"
  echo "  Env: PI_MODEL=$MODEL  PORT=$PORT  CTX_SIZE=$CTX_SIZE"
  exit 0
}

if [[ -z "$PROMPT" || "$PROMPT" == "-h" || "$PROMPT" == "--help" ]]; then usage; fi

echo "[pi-task] killing old pi sessions..."
pkill -9 -f 'node.*pi' 2>/dev/null || true
sleep 1

if ! curl -s --max-time 3 "http://127.0.0.1:$PORT/health" > /dev/null 2>&1; then
  echo "[pi-task] starting server on :$PORT..."
  REASONING=off CTX_SIZE="$CTX_SIZE" PORT="$PORT" \
    nohup "$SCRIPT_DIR/run-server-both.sh" qwen3-8b </dev/null >/tmp/bare-server.log 2>&1 &
  for i in $(seq 1 30); do
    curl -s --max-time 2 "http://127.0.0.1:$PORT/health" > /dev/null 2>&1 && break
    sleep 2
  done
fi

echo "[pi-task] running: $PROMPT"
cd "$(pwd)"
exec pi --provider llama-cpp --model "$MODEL" -a --append-system-prompt "$SCRIPT_DIR/concise-prompt.txt" -p "$PROMPT"

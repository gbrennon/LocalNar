#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODEL="${PI_MODEL:-Qwen3-8B-Q4_K_M}"
PORT="${PORT:-8080}"
CTX_SIZE="${CTX_SIZE:-32768}"
PROMPT="${1:-}"

usage() {
  echo "Usage: pi-task.sh <prompt>"
  echo
  echo "Single-task pi runner with fresh context every invocation."
  echo "Run this script once per task — it kills old pi, ensures the server is up,"
  echo "then runs pi with a clean context window."
  echo
  echo "Env vars:"
  echo "  PI_MODEL   model id              (default: $MODEL)"
  echo "  PORT       server port           (default: $PORT)"
  echo "  CTX_SIZE   context window size   (default: $CTX_SIZE)"
  echo
  echo "Examples:"
  echo "  ./pi-task.sh 'Write a Cargo.toml for a Rust project'"
  echo "  ./pi-task.sh 'Create src/main.rs with an actix-web server'"
  echo "  PI_MODEL=Qwen2.5-7B ./pi-task.sh 'Write a hello world'"
  exit 0
}

if [[ -z "$PROMPT" || "$PROMPT" == "-h" || "$PROMPT" == "--help" ]]; then
  usage
fi

# ── 1. Kill any existing pi sessions ──────────────────────────
echo "[pi-task] killing old pi sessions..."
pkill -9 -f 'node.*pi' 2>/dev/null || true
sleep 1

# ── 2. Ensure server is running ───────────────────────────────
if ! curl -s --max-time 3 "http://127.0.0.1:$PORT/health" > /dev/null 2>&1; then
  echo "[pi-task] server not running on :$PORT, starting..."
  REASONING=off CTX_SIZE="$CTX_SIZE" PORT="$PORT" \
    nohup "$SCRIPT_DIR/run-server.sh" qwen3-8b \
    </dev/null >/tmp/bare-server.log 2>&1 &
  for i in $(seq 1 30); do
    if curl -s --max-time 2 "http://127.0.0.1:$PORT/health" > /dev/null 2>&1; then
      break
    fi
    sleep 2
  done
  if ! curl -s --max-time 2 "http://127.0.0.1:$PORT/health" > /dev/null 2>&1; then
    echo "[pi-task] ERROR: server failed to start on :$PORT" >&2
    exit 1
  fi
  echo "[pi-task] server ready on :$PORT"
fi

# ── 3. Run pi with the prompt ─────────────────────────────────
echo "[pi-task] running: $PROMPT"
echo
cd "$(pwd)"
exec pi --provider llama-cpp --model "$MODEL" -a --append-system-prompt "$SCRIPT_DIR/concise-prompt.txt" -p "$PROMPT"

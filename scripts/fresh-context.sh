#!/usr/bin/env bash
# Kills pi and opens fresh interactive session.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODEL="${1:-Qwen3-8B-Q4_K_M}"

echo "Killing old pi sessions..."
pkill -9 -f 'node.*pi' 2>/dev/null || true
sleep 1

echo "Starting fresh pi with $MODEL..."
cd "$(pwd)"
exec pi --provider llama-cpp --model "$MODEL" --append-system-prompt "$SCRIPT_DIR/concise-prompt.txt"

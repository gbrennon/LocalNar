#!/usr/bin/env bash
# Fast mode: model weights + KV cache both on GPU.
# Usage: ./run-server.sh [model-name]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODELS_DIR="${MODELS_DIR:-$HOME/models}"
LLAMA_BIN="${LLAMA_BIN:-$SCRIPT_DIR/../../llamacpp/build/bin/llama-server}"
LLAMA_BIN_DIR="$(dirname "$LLAMA_BIN")"
PORT="${PORT:-8080}"
OFFLOAD_LAYERS="${OFFLOAD_LAYERS:-99}"
CTX_SIZE="${CTX_SIZE:-8192}"
ALIAS="${ALIAS:-}"
SKIP_CHAT_PARSING="${SKIP_CHAT_PARSING:-}"
CHAT_TEMPLATE="${CHAT_TEMPLATE:-}"
REASONING="${REASONING:-}"

usage() {
  echo "Usage: run-server.sh [model-name]"
  echo ""
  echo "Fast mode: model + KV on GPU."
  echo "  model-name  substring match against ~/models/*.gguf (or run with 'list')"
  echo ""
  echo "Env: CTX_SIZE=${CTX_SIZE} PORT=${PORT} OFFLOAD_LAYERS=${OFFLOAD_LAYERS}"
  echo "     REASONING=${REASONING:-off} SKIP_CHAT_PARSING=${SKIP_CHAT_PARSING:-off} CHAT_TEMPLATE=${CHAT_TEMPLATE:-none}"
}

list_models() { find "$MODELS_DIR" -type f -name '*.gguf' -print | sort; }

if [[ "${1:-}" == "list" || "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  [[ "${1:-}" == "list" ]] && list_models || usage
  exit 0
fi

MODEL_PATH=$(list_models | grep -i "${1:-}" | head -n1 || true)
if [[ -z "$MODEL_PATH" ]]; then
  echo "No model matched '${1:-}'." >&2; list_models >&2; exit 1
fi

if [[ ! -x "$LLAMA_BIN" ]]; then
  echo "llama-server not found at $LLAMA_BIN" >&2; exit 1
fi

ALIAS="${ALIAS:-$(basename "$MODEL_PATH" .gguf)}"
echo "Serving: $MODEL_PATH"
echo "  port: $PORT  gpu layers: $OFFLOAD_LAYERS  ctx: $CTX_SIZE"

export LD_LIBRARY_PATH="${LLAMA_BIN_DIR}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

exec "$LLAMA_BIN" \
  -m "$MODEL_PATH" -ngl "$OFFLOAD_LAYERS" -c "$CTX_SIZE" \
  --alias "$ALIAS" --host 127.0.0.1 --port "$PORT" \
  ${SKIP_CHAT_PARSING:+--skip-chat-parsing} \
  ${CHAT_TEMPLATE:+--chat-template "$CHAT_TEMPLATE"} \
  ${REASONING:+--reasoning "$REASONING"}

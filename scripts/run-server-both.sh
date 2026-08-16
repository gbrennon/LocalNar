#!/usr/bin/env bash
# VRAM + RAM split: model on GPU, KV cache in system RAM.
# Usage: ./run-server-both.sh [model-name]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODELS_DIR="${MODELS_DIR:-$HOME/models}"
LLAMA_BIN="${LLAMA_BIN:-$SCRIPT_DIR/../../llamacpp/build/bin/llama-server}"
LLAMA_BIN_DIR="$(dirname "$LLAMA_BIN")"
PORT="${PORT:-8080}"
CTX_SIZE="${CTX_SIZE:-32768}"

VRAM_TOTAL=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader 2>/dev/null | head -1 | grep -oP '\d+' || echo "12227")
BASELINE=2500
VRAM_FOR_MODEL=$((VRAM_TOTAL - BASELINE))

list_models() { find "$MODELS_DIR" -type f -name '*.gguf' -print | sort; }

if [[ "${1:-}" == "list" || "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  echo "Usage: run-server-both.sh [model-name]"
  echo "  VRAM+RAM split: model on GPU, KV cache in system RAM."
  echo "  VRAM for model: ~${VRAM_FOR_MODEL} MiB  |  KV in RAM"
  echo "  Env: CTX_SIZE=${CTX_SIZE}  PORT=${PORT}  REASONING=${REASONING:-off}"
  [[ "${1:-}" == "list" ]] && list_models
  exit 0
fi

MODEL_PATH=$(list_models | grep -i "${1:-}" | head -n1 || true)
if [[ -z "$MODEL_PATH" ]]; then
  echo "No model matched '${1:-}'." >&2; list_models >&2; exit 1
fi

MODEL_SIZE_MiB=$(($(stat -c '%s' "$MODEL_PATH") / 1048576))
ALIAS="${ALIAS:-$(basename "$MODEL_PATH" .gguf)}"

if [ "$MODEL_SIZE_MiB" -lt "$((VRAM_FOR_MODEL * 80 / 100))" ]; then
  OFFLOAD_LAYERS=99
else
  OFFLOAD_LAYERS=$((99 * VRAM_FOR_MODEL / MODEL_SIZE_MiB))
  [ "$OFFLOAD_LAYERS" -gt 99 ] && OFFLOAD_LAYERS=99
  [ "$OFFLOAD_LAYERS" -lt 10 ] && OFFLOAD_LAYERS=10
fi

echo "Model: $ALIAS (${MODEL_SIZE_MiB} MiB)  |  ctx: $CTX_SIZE  |  KV: CPU RAM"
echo "VRAM: ${VRAM_TOTAL} MiB total  |  ~${VRAM_FOR_MODEL} MiB for model  |  layers: $OFFLOAD_LAYERS GPU"

export LD_LIBRARY_PATH="${LLAMA_BIN_DIR}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$LLAMA_BIN" \
  -m "$MODEL_PATH" -ngl "$OFFLOAD_LAYERS" -c "$CTX_SIZE" \
  --alias "$ALIAS" --host 127.0.0.1 --port "$PORT" \
  --no-kv-offload ${REASONING:+--reasoning "$REASONING"} \
  ${SKIP_CHAT_PARSING:+--skip-chat-parsing} \
  ${CHAT_TEMPLATE:+--chat-template "$CHAT_TEMPLATE"}

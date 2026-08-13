#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODELS_DIR="${MODELS_DIR:-$HOME/models}"
LLAMA_BIN="${LLAMA_BIN:-$SCRIPT_DIR/../llamacpp/build/bin/llama-server}"
LLAMA_BIN_DIR="$(dirname "$LLAMA_BIN")"
PORT="${PORT:-8080}"
OFFLOAD_LAYERS="${OFFLOAD_LAYERS:-99}"
CTX_SIZE="${CTX_SIZE:-8192}"
ALIAS="${ALIAS:-}"
SKIP_CHAT_PARSING="${SKIP_CHAT_PARSING:-}"
CHAT_TEMPLATE="${CHAT_TEMPLATE:-}"
REASONING="${REASONING:-}"  # set to 'off' for Qwen3 models (thinking mode eats output tokens)

usage() {
  echo "Usage: run-server.sh [model-name]"
  echo
  echo "Starts llama-server against a GGUF model found under $MODELS_DIR."
  echo
  echo "Options:"
  echo "  model-name   substring of the model file name (default: first model found)."
  echo "               Run with 'list' to show available models."
  echo
  echo "Environment overrides:"
  echo "  MODELS_DIR    model search directory   (default: $MODELS_DIR)"
  echo "  PORT          listen port              (default: $PORT)"
  echo "  OFFLOAD_LAYERS GPU layers, 99 = all     (default: $OFFLOAD_LAYERS)"
  echo "  CTX_SIZE      context size             (default: $CTX_SIZE)"
  echo "  ALIAS         model id exposed as      (default: filename without .gguf)"
  echo "  SKIP_CHAT_PARSING set to 1 to skip autoparser  (default: off)"
  echo "  CHAT_TEMPLATE    override jinja template  (default: model's built-in)"
}

list_models() {
  find "$MODELS_DIR" -type f -name '*.gguf' -print | sort
}

if [[ "${1:-}" == "list" || "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  if [[ "${1:-}" == "list" ]]; then
    list_models
  else
    usage
  fi
  exit 0
fi

if [[ $# -gt 1 ]]; then
  usage
  exit 1
fi

MODEL_PATH=""
if [[ $# -eq 1 ]]; then
  MODEL_PATH=$(list_models | grep -i "$1" | head -n1 || true)
  if [[ -z "$MODEL_PATH" ]]; then
    echo "No model in $MODELS_DIR matched '$1'." >&2
    echo "Available models:" >&2
    list_models >&2
    exit 1
  fi
else
  MODEL_PATH=$(list_models | head -n1 || true)
  if [[ -z "$MODEL_PATH" ]]; then
    echo "No .gguf models found under $MODELS_DIR." >&2
    exit 1
  fi
fi

if [[ ! -x "$LLAMA_BIN" ]]; then
  echo "llama-server not found at $LLAMA_BIN." >&2
  echo "Point LLAMA_BIN at your build, or build the sibling repo first:" >&2
  echo "  cmake -B ../llamacpp/build -DGGML_VULKAN=ON -DCMAKE_BUILD_TYPE=Release" >&2
  echo "  cmake --build ../llamacpp/build --target llama-server -j4" >&2
  exit 1
fi

echo "Serving: $MODEL_PATH"
echo "  port      : $PORT"
echo "  gpu layers: $OFFLOAD_LAYERS"
echo "  ctx size  : $CTX_SIZE"

if [[ -z "$ALIAS" ]]; then
  ALIAS="$(basename "$MODEL_PATH" .gguf)"
fi

export LD_LIBRARY_PATH="${LLAMA_BIN_DIR}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

case "${KV_OFFLOAD:-1}" in
  0) KV_FLAG="--no-kv-offload" ;;
  *) KV_FLAG="--kv-offload" ;;
esac

exec "$LLAMA_BIN" \
  -m "$MODEL_PATH" \
  -ngl "$OFFLOAD_LAYERS" \
  -c "$CTX_SIZE" \
  --alias "$ALIAS" \
  --host 127.0.0.1 \
  --port "$PORT" \
  ${SKIP_CHAT_PARSING:+--skip-chat-parsing} \
  ${CHAT_TEMPLATE:+--chat-template "$CHAT_TEMPLATE"} \
  "$KV_FLAG" \
  ${REASONING:+--reasoning "$REASONING"}

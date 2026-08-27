#!/usr/bin/env bash
# Fast mode: model weights + KV cache both on GPU.
# Usage: ./run-server.sh [model-name]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)" || exit 1
readonly SCRIPT_DIR
MODELS_DIR="${MODELS_DIR:-$HOME/models}" || exit 1
readonly MODELS_DIR
LLAMA_BIN="${LLAMA_BIN:-$SCRIPT_DIR/../llama.cpp/build/bin/llama-server}"
readonly LLAMA_BIN
LLAMA_BIN_DIR="$(dirname "$LLAMA_BIN")" || exit 1
readonly LLAMA_BIN_DIR
readonly PORT="${PORT:-8080}"
readonly HOST="${HOST:-0.0.0.0}"
readonly OFFLOAD_LAYERS="${OFFLOAD_LAYERS:-99}"
readonly CTX_SIZE="${CTX_SIZE:-8192}"
readonly ALIAS="${ALIAS:-}"
readonly SKIP_CHAT_PARSING="${SKIP_CHAT_PARSING:-}"
readonly CHAT_TEMPLATE="${CHAT_TEMPLATE:-}"
readonly REASONING="${REASONING:-}"

die() {
  echo "$@" >&2
  exit 1
}

validate_llama_binary() {
  [[ -x "$LLAMA_BIN" ]] || die "llama-server not found at $LLAMA_BIN"
}

validate_model_path() {
  local model_path="$1"
  [[ -n "$model_path" ]] || die "No model matched '${1:-}'. Available models:" "$(list_models)"
}

list_models() {
  find "$MODELS_DIR" -type f -name '*.gguf' -print | sort
}

find_model() {
  local query="${1:-}"
  list_models | grep -i "$query" | head -n1 || true
}

get_lan_ip() {
  local ip
  ip="$(ip route get 1.1.1.1 2>/dev/null | grep -oP 'src \K[0-9.]+' || true)"
  if [[ -z "$ip" ]]; then
    ip="$(hostname -I 2>/dev/null | awk '{print $1}' || true)"
  fi
  echo "${ip:-127.0.0.1}"
}

setup_library_path() {
  export LD_LIBRARY_PATH="${LLAMA_BIN_DIR}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
}

print_server_info() {
  local model_path="$1"
  local lan_ip="$2"
  local alias="$3"

  echo "Serving: $model_path"
  echo "  alias:  $alias"
  echo "  port: $PORT  gpu layers: $OFFLOAD_LAYERS  ctx: $CTX_SIZE"
  echo "  local:   http://127.0.0.1:$PORT"
  echo "  network: http://$lan_ip:$PORT  (reachable by other machines)"
}

build_server_args() {
  local model_path="$1"
  local alias="$2"

  local args=(
    -m "$model_path"
    -ngl "$OFFLOAD_LAYERS"
    -c "$CTX_SIZE"
    --alias "$alias"
    --host "$HOST"
    --port "$PORT"
  )

  [[ -n "$SKIP_CHAT_PARSING" ]] && args+=(--skip-chat-parsing)
  [[ -n "$CHAT_TEMPLATE" ]] && args+=(--chat-template "$CHAT_TEMPLATE")
  [[ -n "$REASONING" ]] && args+=(--reasoning "$REASONING")

  printf '%s\n' "${args[@]}"
}

start_server() {
  local model_path="$1"
  local alias="$2"

  setup_library_path
  print_server_info "$model_path" "$(get_lan_ip)" "$alias"

  local -a args
  mapfile -t args < <(build_server_args "$model_path" "$alias")
  exec "$LLAMA_BIN" "${args[@]}"
}

print_usage() {
  cat <<EOF
Usage: run-server.sh [model-name]

Fast mode: model + KV on GPU.
  model-name  substring match against $MODELS_DIR/*.gguf (or run with 'list')

Env: CTX_SIZE=${CTX_SIZE} PORT=${PORT} HOST=${HOST} OFFLOAD_LAYERS=${OFFLOAD_LAYERS}
     REASONING=${REASONING:-off} SKIP_CHAT_PARSING=${SKIP_CHAT_PARSING:-off} CHAT_TEMPLATE=${CHAT_TEMPLATE:-none}
EOF
}

handle_help_or_list() {
  local arg="${1:-}"

  case "$arg" in
    list)
      list_models
      exit 0
      ;;
    -h | --help)
      print_usage
      exit 0
      ;;
  esac
}

main() {
  handle_help_or_list "${1:-}"

  validate_llama_binary

  local model_path
  model_path="$(find_model "${1:-}")"
  validate_model_path "$model_path"

  local alias="${ALIAS:-$(basename "$model_path" .gguf)}"
  start_server "$model_path" "$alias"
}

main "$@"

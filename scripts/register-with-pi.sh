#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_NAME="$(basename "$SCRIPT_DIR")"
PI_BIN="$(command -v pi || true)"
PI_SESSIONS_DIR="${PI_SESSIONS_DIR:-$HOME/.pi/agent/sessions}"

usage() {
  echo "Usage: register-with-pi.sh [project-name] [pi args...]"
  echo
  echo "Registers this repo with the pi coding agent under a stable project session."
  echo "Pi has no 'register' command; a project is registered by creating a session with"
  echo "an exact project id (pi --session-id <name>). This script does exactly that."
  echo
  echo "Modes:"
  echo "  register-with-pi.sh                  register as '$PROJECT_NAME' and open pi"
  echo "  register-with-pi.sh <name> ...       register under a custom name, forward args to pi"
  echo "  register-with-pi.sh list             list existing pi project buckets matching the name"
  echo "  register-with-pi.sh --dry-run        print what would run without executing"
  echo "  register-with-pi.sh -h|--help        show this help"
  echo
  echo "Extra args after the name are passed through to pi (e.g. --provider, -p, ...)."
}

list_buckets() {
  local pattern
  pattern="$(printf '%s' "$PROJECT_NAME" | tr ' /\\:' '-')"
  if [[ ! -d "$PI_SESSIONS_DIR" ]]; then
    echo "No pi sessions dir at $PI_SESSIONS_DIR."
    return 0
  fi
  find "$PI_SESSIONS_DIR" -maxdepth 1 -type d -name "*${pattern}*" -printf '%f\n' | sort
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ "${1:-}" == "list" ]]; then
  echo "Pi project buckets matching '$PROJECT_NAME' under $PI_SESSIONS_DIR:"
  list_buckets
  exit 0
fi

if [[ -n "${1:-}" && "${1:-}" != --* ]]; then
  PROJECT_NAME="$1"
  shift
fi

if [[ -z "$PI_BIN" ]]; then
  echo "pi not found on PATH. Install it first (e.g. npm i -g @earendil-works/pi-coding-agent)." >&2
  exit 1
fi

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
  shift
fi

FORWARD_ARGS=("$@")

if [[ "$DRY_RUN" == true ]]; then
  echo "Would register '$PROJECT_NAME' for repo at $SCRIPT_DIR"
  echo "Command: (cd $SCRIPT_DIR && $PI_BIN --session-id $PROJECT_NAME ${FORWARD_ARGS[*]:-})"
  echo "Existing matching buckets:"
  list_buckets
  exit 0
fi

cd "$SCRIPT_DIR"
echo "Registering '$PROJECT_NAME' with pi (cwd: $SCRIPT_DIR)"
exec "$PI_BIN" --session-id "$PROJECT_NAME" "${FORWARD_ARGS[@]}"

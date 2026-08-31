#!/usr/bin/env bash
set -euo pipefail

readonly FORBIDDEN_PATTERN='llama\.cpp'

reject_forbidden_mention_in_tracked_files() {
  local matches=""
  matches="$(git grep -in -e "$FORBIDDEN_PATTERN" -- . \
    ':(exclude)scripts/check_no_llama_cpp.sh' \
    ':(exclude)scripts/README.md' \
    ':(exclude)README.md' || true)"
  if [ -n "$matches" ]; then
    echo "ERROR: tracked files mention llama.cpp:" >&2
    echo "$matches" >&2
    exit 1
  fi
}

reject_tracked_gitmodules() {
  if git ls-files --error-unmatch .gitmodules >/dev/null 2>&1; then
    echo "ERROR: .gitmodules is tracked; submodules are not allowed in this repository." >&2
    exit 1
  fi
}

reject_tracked_gitlink_entries() {
  local gitlinks=""
  gitlinks="$(git ls-files -s | awk '$1 == 160000 { print $4 }')"
  if [ -n "$gitlinks" ]; then
    echo "ERROR: tracked gitlink entries (mode 160000) found; submodules are not allowed:" >&2
    echo "$gitlinks" >&2
    exit 1
  fi
}

validate_no_llama_cpp() {
  cd "$(git rev-parse --show-toplevel)"
  echo "Checking tracked files for llama.cpp mentions, .gitmodules, and gitlink entries"
  reject_forbidden_mention_in_tracked_files
  reject_tracked_gitmodules
  reject_tracked_gitlink_entries
  echo "llama.cpp check: PASS"
}

validate_no_llama_cpp

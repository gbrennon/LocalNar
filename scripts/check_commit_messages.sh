#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

readonly CONVENTIONAL_COMMIT_TYPES='feat|fix|docs|style|refactor|perf|test|chore|revert|ci|build'
readonly MERGE_COMMIT_PATTERN='^Merge .+'
readonly FALLBACK_COMMIT_COUNT=20

collect_commit_messages() {
  local commit_range
  commit_range="$(resolve_commit_range)"

  if [ -n "$commit_range" ]; then
    git log --format=%s "$commit_range"
  else
    git log --format=%s -n "$FALLBACK_COMMIT_COUNT"
  fi
}

commit_message_is_valid() {
  local commit_message="$1"
  local conventional_pattern="^(${CONVENTIONAL_COMMIT_TYPES})(\(.+\))?!?: ?.+"

  grep -qE "(${conventional_pattern}|${MERGE_COMMIT_PATTERN})" <<<"$commit_message"
}

abort_if_any_commit_message_violates_conventional_commits() {
  local commit_messages="$1"
  local invalid_commit_messages=()
  local commit_message

  while IFS= read -r commit_message; do
    if [ -z "$commit_message" ]; then
      continue
    fi

    if ! commit_message_is_valid "$commit_message"; then
      invalid_commit_messages+=("$commit_message")
    fi
  done <<<"$commit_messages"

  if [ "${#invalid_commit_messages[@]}" -eq 0 ]; then
    return
  fi

  for commit_message in "${invalid_commit_messages[@]}"; do
    echo "ERROR: commit message does not follow conventional commits: '$commit_message'" >&2
  done
  echo "Allowed types: ${CONVENTIONAL_COMMIT_TYPES//|/, }" >&2
  exit 1
}

validate_commit_messages() {
  local commit_messages
  commit_messages="$(collect_commit_messages)"

  if [ -z "$commit_messages" ]; then
    echo "No commits to check"
    return
  fi

  abort_if_any_commit_message_violates_conventional_commits "$commit_messages"
  echo "Commit message check: PASS"
}

validate_commit_messages

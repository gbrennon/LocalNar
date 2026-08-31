#!/usr/bin/env bash
set -euo pipefail

# Resolves the branch under validation.
#
# Contract: prefers the pull request head branch, falls back to the pushed ref
# with its `refs/heads/` prefix stripped, and finally to the checked out branch.
# Returns an empty string when no branch can be determined.
resolve_current_branch_name() {
  local head_ref="${CI_HEAD_REF:-${GITHUB_HEAD_REF:-}}"
  local full_ref="${CI_REF:-${GITHUB_REF:-}}"

  if [ -n "$head_ref" ]; then
    echo "$head_ref"
  elif [ -n "$full_ref" ]; then
    echo "${full_ref#refs/heads/}"
  else
    git rev-parse --abbrev-ref HEAD 2>/dev/null || echo ""
  fi
}

# Resolves the git revision range holding the commits under validation.
#
# Contract: returns `origin/<base>..origin/<head>` for pull requests whose refs
# are both fetched, `HEAD~1..HEAD` when the parent commit is available, and an
# empty string when neither range can be resolved.
resolve_commit_range() {
  local base_ref="${CI_BASE_REF:-${GITHUB_BASE_REF:-}}"
  local head_ref="${CI_HEAD_REF:-${GITHUB_HEAD_REF:-}}"

  if [ -n "$base_ref" ] && [ -n "$head_ref" ] &&
    git rev-parse --verify --quiet "origin/${base_ref}" >/dev/null &&
    git rev-parse --verify --quiet "origin/${head_ref}" >/dev/null; then
    echo "origin/${base_ref}..origin/${head_ref}"
  elif git rev-parse --verify --quiet HEAD~1 >/dev/null; then
    echo "HEAD~1..HEAD"
  else
    echo ""
  fi
}

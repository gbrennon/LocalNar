#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

readonly VALID_BRANCH_NAME_PATTERN='^(main|master|develop|feature/.+|feat/.+|bugfix/.+|fix/.+|hotfix/.+|release/.+|chore/.+)$'

abort_if_branch_name_is_unresolved() {
  local branch_name="$1"

  if [ -z "$branch_name" ]; then
    echo "ERROR: could not resolve the current branch name." >&2
    exit 1
  fi
}

abort_if_branch_name_violates_naming_convention() {
  local branch_name="$1"

  if [[ ! "$branch_name" =~ $VALID_BRANCH_NAME_PATTERN ]]; then
    echo "ERROR: branch name '$branch_name' does not follow the naming convention." >&2
    echo "Allowed prefixes: feature/, feat/, bugfix/, fix/, hotfix/, release/, chore/" >&2
    exit 1
  fi
}

validate_current_branch_name() {
  local branch_name
  branch_name="$(resolve_current_branch_name)"

  echo "Branch: $branch_name"
  abort_if_branch_name_is_unresolved "$branch_name"
  abort_if_branch_name_violates_naming_convention "$branch_name"
  echo "Branch name check: PASS"
}

validate_current_branch_name

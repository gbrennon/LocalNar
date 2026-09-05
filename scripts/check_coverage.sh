#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=scripts/lib/common.sh
source "$(dirname "$0")/lib/common.sh"

readonly COVERAGE_THRESHOLD="${COVERAGE_THRESHOLD:-}"

resolve_crate_package() {
  local target="$1"
  [ -z "$target" ] && return 0

  local packages
  packages=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[].name')

  for pkg in $packages; do
    if [ "$target" = "$pkg" ]; then
      echo "$pkg"
      return 0
    fi
    if [ "localnar-$target" = "$pkg" ]; then
      echo "$pkg"
      return 0
    fi
  done

  echo "ERROR: Unrecognized crate/package '${target}'." >&2
  echo "Available workspace packages:" >&2
  for pkg in $packages; do
    local short="${pkg#localnar-}"
    if [ "$short" != "$pkg" ]; then
      echo "  - ${pkg} (or: ${short})" >&2
    else
      echo "  - ${pkg}" >&2
    fi
  done
  return 1
}

run_coverage_and_emit_json() {
  local package_name="$1"
  rm -f cov.json
  if [ -n "$package_name" ]; then
    echo "Running cargo-llvm-cov for package '${package_name}' (generating JSON report)..."
    cargo llvm-cov --package "$package_name" --json --output-path cov.json || true
  else
    echo "Running cargo-llvm-cov across workspace (generating JSON report)..."
    cargo llvm-cov --workspace --json --output-path cov.json || true
  fi
}

main() {
  local target="${1:-}"
  local package_name=""

  if [ -n "$target" ]; then
    if ! package_name=$(resolve_crate_package "$target"); then
      exit 1
    fi
  fi

  run_coverage_and_emit_json "$package_name"
  abort_if_coverage_json_is_missing
  print_coverage_table

  if [ -n "$COVERAGE_THRESHOLD" ]; then
    abort_if_line_coverage_is_below_threshold "$COVERAGE_THRESHOLD"
  fi
}

main "$@"

#!/usr/bin/env bash
set -euo pipefail

validate_release_level() {
  local release_level="$1"
  local semver_pattern='^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'

  case "$release_level" in
    patch|minor|major)
      return 0
      ;;
    *)
      if [[ "$release_level" =~ $semver_pattern ]]; then
        return 0
      fi
      echo "ERROR: Invalid release level '$release_level'. Expected 'patch', 'minor', 'major', or explicit semver (e.g. 1.2.3)." >&2
      exit 1
      ;;
  esac
}

validate_environment() {
  local environment_name="$1"

  case "$environment_name" in
    staging)
      echo "staging"
      ;;
    production|prod)
      echo "production"
      ;;
    *)
      echo "ERROR: Invalid environment '$environment_name'. Expected 'staging' or 'production'." >&2
      exit 1
      ;;
  esac
}

validate_execute_flag() {
  local execute_value="$1"

  case "$execute_value" in
    true|1|yes|execute|false|0|no|dry-run|"")
      return 0
      ;;
    *)
      echo "ERROR: Invalid execute flag '$execute_value'. Expected 'true', 'false', or 'dry-run'." >&2
      exit 1
      ;;
  esac
}

is_execute_mode() {
  local execute_value="$1"

  case "$execute_value" in
    true|1|yes|execute)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

abort_if_working_tree_is_dirty() {
  if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Git working directory contains uncommitted changes. Working tree must be clean before release." >&2
    exit 1
  fi
}

commit_workspace_version() {
  local new_version="$1"

  if [ -n "$(git status --porcelain Cargo.toml)" ]; then
    echo "Committing workspace version bump to ${new_version}..."
    git add Cargo.toml
    if [ -f Cargo.lock ] && [ -n "$(git status --porcelain Cargo.lock)" ]; then
      git add Cargo.lock
    fi
    git commit -m "chore(release): bump version to ${new_version}"
  fi
}

find_matching_package() {
  local target_crate="$1"
  local package_names
  package_names="$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[].name')"

  for package_name in $package_names; do
    if [ "$package_name" = "$target_crate" ] || [ "$package_name" = "localnar-${target_crate}" ]; then
      echo "$package_name"
      return 0
    fi
  done

  return 1
}

resolve_target_crates() {
  local target_input="$1"
  local default_crate_order="localnar-domain localnar-application localnar-infrastructure localnar-presentation localnar"

  if [ -z "$target_input" ] || [ "$target_input" = "all" ] || [ "$target_input" = "all crates" ]; then
    echo "$default_crate_order"
    return 0
  fi

  local resolved_crate
  if resolved_crate="$(find_matching_package "$target_input")"; then
    echo "$resolved_crate"
    return 0
  fi

  echo "ERROR: Unknown target crate '$target_input'." >&2
  echo "Valid options: all, localnar, domain, application, infrastructure, presentation" >&2
  exit 1
}

calculate_next_version() {
  local current_version="$1"
  local release_level="$2"
  local semver_pattern='^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'

  if [[ "$release_level" =~ $semver_pattern ]]; then
    echo "$release_level"
    return 0
  fi

  local major minor patch
  IFS='.' read -r major minor patch <<< "$current_version"

  case "$release_level" in
    patch)
      echo "${major}.${minor}.$((patch + 1))"
      ;;
    minor)
      echo "${major}.$((minor + 1)).0"
      ;;
    major)
      echo "$((major + 1)).0.0"
      ;;
  esac
}

read_workspace_version() {
  sed -n -E 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -n 1
}

apply_workspace_version() {
  local new_version="$1"

  sed -i "s/^version = \".*\"/version = \"${new_version}\"/" Cargo.toml
  sed -i -E "s/^(localnar-(domain|application|infrastructure|presentation) = \\{ path = \"[^\"]+\", version = \")[^\"]+/\\1${new_version}/" Cargo.toml
}

is_crate_version_published() {
  local crate_name="$1"
  local target_version="$2"
  local index_base="$3"
  local prefix_one="${crate_name:0:2}"
  local prefix_two="${crate_name:2:2}"
  local index_url="${index_base}/${prefix_one}/${prefix_two}/${crate_name}"
  local index_content

  index_content="$(curl -s -f "$index_url" 2>/dev/null || true)"
  if echo "$index_content" | grep -q "\"vers\":\"${target_version}\""; then
    return 0
  fi
  return 1
}

publish_single_crate() {
  local crate_name="$1"
  local target_version="$2"
  local index_base="$3"
  local execute_flag="$4"
  shift 4
  local registry_args=("$@")

  if is_crate_version_published "$crate_name" "$target_version" "$index_base"; then
    echo "Notice: ${crate_name} version ${target_version} is already published on ${index_base}, skipping."
    return 0
  fi

  local dry_run_flag=()
  if ! is_execute_mode "$execute_flag"; then
    dry_run_flag=("--dry-run")
  fi

  echo "Publishing ${crate_name} version ${target_version}..."
  cargo publish -p "$crate_name" "${registry_args[@]}" --no-verify "${dry_run_flag[@]}"
}

publish_all_crates() {
  local target_crates="$1"
  local publish_version="$2"
  local index_base="$3"
  local execute_flag="$4"
  shift 4
  local registry_args=("$@")

  for crate_name in $target_crates; do
    publish_single_crate "$crate_name" "$publish_version" "$index_base" "$execute_flag" "${registry_args[@]}"
  done
}

configure_registry_environment() {
  local environment_name="$1"

  if [ "$environment_name" = "staging" ]; then
    export CARGO_REGISTRIES_STAGING_INDEX="sparse+https://index.staging.crates.io/"
    if [ -n "${CRATES_IO_STAGING_TOKEN:-}" ]; then
      cargo login --registry staging "$CRATES_IO_STAGING_TOKEN"
    fi
    return 0
  fi

  if [ -n "${CRATES_IO_TOKEN:-}" ]; then
    cargo login "$CRATES_IO_TOKEN"
  fi
}

execute_preflight_verification() {
  echo "Running pre-flight workspace verification..."
  ./scripts/verify.sh
}

main() {
  local release_level="${1:-patch}"
  local environment_input="${2:-staging}"
  local target_crate_input="${3:-all}"
  local execute_input="${4:-false}"

  abort_if_working_tree_is_dirty
  validate_release_level "$release_level"
  validate_execute_flag "$execute_input"

  local normalized_environment
  normalized_environment="$(validate_environment "$environment_input")"
  local target_crates
  target_crates="$(resolve_target_crates "$target_crate_input")"

  execute_preflight_verification

  local current_version next_version
  current_version="$(read_workspace_version)"
  next_version="$(calculate_next_version "$current_version" "$release_level")"

  local publish_version="$current_version"
  if is_execute_mode "$execute_input"; then
    echo "Bumping workspace version: ${current_version} -> ${next_version}"
    apply_workspace_version "$next_version"
    commit_workspace_version "$next_version"
    publish_version="$next_version"
  else
    echo "Dry-run mode: skipping version bump on disk (would bump ${current_version} -> ${next_version})."
  fi

  configure_registry_environment "$normalized_environment"

  local index_base registry_args=()
  if [ "$normalized_environment" = "staging" ]; then
    index_base="https://index.staging.crates.io"
    registry_args=("--registry" "staging")
  else
    index_base="https://index.crates.io"
  fi

  publish_all_crates "$target_crates" "$publish_version" "$index_base" "$execute_input" "${registry_args[@]}"

  if is_execute_mode "$execute_input"; then
    echo "Release completed successfully for version ${next_version} (${normalized_environment})."
  else
    echo "Dry-run release verification completed for version ${next_version} (${normalized_environment}). Pass execute=true to publish."
  fi
}

main "$@"

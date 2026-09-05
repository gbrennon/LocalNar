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

# Coverage functions for cargo-llvm-cov reports

coverage_json_exists() {
  [ -f cov.json ]
}

abort_if_coverage_json_is_missing() {
  if ! coverage_json_exists; then
    echo "ERROR: cov.json not found. cargo-llvm-cov failed to produce JSON output." >&2
    exit 1
  fi
}

extract_coverage_totals_from_json() {
  lines_count=$(jq -r '.data[0].totals.lines.count' cov.json)
  lines_covered=$(jq -r '.data[0].totals.lines.covered' cov.json)
  lines_percent=$(jq -r '.data[0].totals.lines.percent' cov.json)
  functions_percent=$(jq -r '.data[0].totals.functions.percent' cov.json)
  regions_percent=$(jq -r '.data[0].totals.regions.percent' cov.json)

  lines_percent=${lines_percent:-0}
  functions_percent=${functions_percent:-0}
  regions_percent=${regions_percent:-0}

  export lines_count lines_covered lines_percent functions_percent regions_percent
}

normalize_path() {
  local path="$1"
  if [[ "$path" == *"crates/"* ]]; then
    path="crates/${path#*crates/}"
  elif [[ "$path" == *"src/"* ]]; then
    path="src/${path#*src/}"
  fi
  echo "$path"
}

extract_missing_lines() {
  local file_path="$1"

  jq -r --arg fp "$file_path" '
    .data[0].files[]
    | select(.filename == $fp)
    | .segments as $segs
    | [ range(0; $segs | length)
        | . as $i
        | $segs[$i]
        | select(.[2] == 0 and .[3] == false)
        | { start: .[0], end: ($segs[$i+1] // .[0:1])[0] }
      ]
    | group_by(.start)
    | map(.[0])
    | map(
        if .start == .end then (.start | tostring)
        else "\(.start)-\(.end)"
        end
      )
    | join(", ")
  ' cov.json
}

print_coverage_table() {
  extract_coverage_totals_from_json

  printf "\n"

  local tmp_rows
  tmp_rows=$(mktemp)

  jq -r '
    .data[0].files[]
    | select(.summary.lines.count > 0)
    | [
        .filename,
        (.summary.lines.count | tostring),
        ((.summary.lines.count - .summary.lines.covered) | tostring),
        ((.summary.lines.covered / .summary.lines.count * 100) | tostring)
      ]
    | @tsv
  ' cov.json | while IFS=$'\t' read -r raw_path stmts miss pct; do
    local norm
    norm=$(normalize_path "$raw_path")
    printf '%s\t%s\t%s\t%s\t%s\n' "$norm" "$stmts" "$miss" "$pct" "$raw_path"
  done | sort > "$tmp_rows"

  local max_name_len
  max_name_len=$(awk -F'\t' '{print length($1)}' "$tmp_rows" | sort -n | tail -1)
  max_name_len=${max_name_len:-4}
  local name_col=$(( max_name_len > 4 ? max_name_len : 4 ))

  local max_missing_len=7  # minimum width for "Missing" header
  while IFS=$'\t' read -r norm stmts miss pct raw_path; do
    if [ "$miss" -gt 0 ]; then
      local mlines
      mlines=$(extract_missing_lines "$raw_path")
      local mlen=${#mlines}
      [ "$mlen" -gt "$max_missing_len" ] && max_missing_len=$mlen
    fi
  done < "$tmp_rows"

  if [ "$max_missing_len" -gt 80 ]; then
    max_missing_len=80
  fi

  local sep
  sep=$(printf '%*s' $(( name_col + 28 + max_missing_len )) '' | tr ' ' '-')

  printf "%-${name_col}s  %6s  %4s  %6s  %-${max_missing_len}s\n" \
    "Name" "Stmts" "Miss" "Cover" "Missing"
  echo "$sep"

  while IFS=$'\t' read -r norm stmts miss pct raw_path; do
    local missing_lines=""
    if [ "$miss" -gt 0 ]; then
      missing_lines=$(extract_missing_lines "$raw_path")

      if [ "${#missing_lines}" -gt "$max_missing_len" ]; then
        missing_lines="${missing_lines:0:$((max_missing_len-4))} ..."
      fi
    fi
    printf "%-${name_col}s  %6s  %4s  %5.1f%%  %-${max_missing_len}s\n" \
      "$norm" "$stmts" "$miss" "$pct" "$missing_lines"
  done < "$tmp_rows"

  echo "$sep"
  printf "%-${name_col}s  %6s  %4s  %5.1f%%\n" \
    "TOTAL" "$lines_count" "$(( lines_count - lines_covered ))" "$lines_percent"

  printf "\n"
  printf "  Functions: %.1f%%\n" "$functions_percent"
  printf "  Regions:   %.1f%%\n" "$regions_percent"

  rm -f "$tmp_rows"
}

abort_if_line_coverage_is_below_threshold() {
  extract_coverage_totals_from_json
  local threshold="$1"
  local passes
  passes=$(awk -v p="$lines_percent" -v t="$threshold" \
    'BEGIN{ if (p+0 >= t+0) print 1; else print 0 }')
  if [ "$passes" -eq 1 ]; then
    printf "Coverage check: PASS (>= %s%%)\n" "$threshold"
  else
    printf "Coverage check: FAIL (< %s%%)\n" "$threshold"
    exit 1
  fi
}

#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: publish_crates.sh <staging|production> <api-key>" >&2
    exit 1
fi

mode="$1"
api_key="$2"

if [[ "$mode" != "staging" && "$mode" != "production" ]]; then
    echo "Invalid mode '${mode}': expected 'staging' or 'production'" >&2
    exit 1
fi

if [[ -z "$api_key" ]]; then
    echo "API key must not be empty" >&2
    exit 1
fi

case "$mode" in
    staging)
        if [[ -z "${PR_NUMBER:-}" || -z "${DEV_COUNTER:-}" ]]; then
            echo "PR_NUMBER and DEV_COUNTER are required for staging publishes" >&2
            exit 1
        fi
        export CARGO_REGISTRIES_STAGING_INDEX="sparse+https://index.staging.crates.io"
        printf '%s' "$api_key" | cargo login --registry staging
        index_base="https://index.staging.crates.io"
        registry_flag=(--registry staging)
        ;;
    production)
        printf '%s' "$api_key" | cargo login
        index_base="https://index.crates.io"
        registry_flag=()
        ;;
esac

base_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)

if [[ "$mode" == "staging" ]]; then
    publish_version="${base_version}-pr.${PR_NUMBER}.dev-${DEV_COUNTER}"
    sed -i "s/^version = \".*\"/version = \"${publish_version}\"/" Cargo.toml
    sed -i -E "s/^(localnar-(domain|application|infrastructure|presentation) = \{ path = \"[^\"]+\", version = \")[^\"]+/\1${publish_version}/" Cargo.toml
else
    publish_version="$base_version"
fi

crate_names=(
    localnar-domain
    localnar-application
    localnar-infrastructure
    localnar-presentation
    localnar
)

last_index=$((${#crate_names[@]} - 1))
for index in "${!crate_names[@]}"; do
    name="${crate_names[$index]}"
    index_url="${index_base}/${name:0:2}/${name:2:2}/${name}"
    if metadata=$(curl -fsSL "$index_url" 2>/dev/null) && grep -q "\"vers\":\"${publish_version}\"" <<<"$metadata"; then
        echo "Skipping ${name} ${publish_version} (already published)"
        continue
    fi
    cargo publish -p "$name" "${registry_flag[@]}"
    if (( index < last_index )); then
        sleep 30
    fi
done

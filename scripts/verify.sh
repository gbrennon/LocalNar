#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

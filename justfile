default:
	@just --list

# Run tests with coverage table (workspace-wide or for a single crate)
# Usage: just test [crate-name]
test crate='':
    ./scripts/check_coverage.sh {{crate}}

# Run tests without coverage (faster, for local development)
test-local crate='':
    cargo test {{ if crate != "" { "-p " + crate } else { "--workspace" } }}

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

lint-fix:
    cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged

build:
    cargo build --workspace

lint-scripts:
    shellcheck --external-sources -S info scripts/lib/common.sh scripts/check_branch_name.sh scripts/check_commit_messages.sh scripts/check_no_llama_cpp.sh scripts/verify.sh scripts/check_coverage.sh

lint-workflows:
    actionlint -config-file .actionlint.yaml .forgejo/workflows/*.yml

install-hooks:
    lefthook install

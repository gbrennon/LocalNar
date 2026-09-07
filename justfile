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

release level='patch' env='staging' crate='all' execute='false':
    ./scripts/release.sh {{level}} {{env}} "{{crate}}" {{execute}}

lint-scripts:
    find scripts -type f -name '*.sh' -exec shellcheck --external-sources -S info {} +

lint-workflows:
    actionlint -config-file .actionlint.yaml .forgejo/workflows/*.yml

install-hooks:
    lefthook install

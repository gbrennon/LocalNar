test:
    cargo test --workspace

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
    shellcheck --external-sources -S info scripts/lib/common.sh scripts/check_branch_name.sh scripts/check_commit_messages.sh scripts/check_no_llama_cpp.sh

lint-workflows:
    actionlint -config-file .actionlint.yaml .forgejo/workflows/*.yml

install-hooks:
    lefthook install

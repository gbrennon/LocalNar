# LocalNar Scripts

Repository checks used by CI and by the lefthook pre-commit hooks. Nothing here
downloads, serves, or runs a model - managing models is the application's job.

| Script | Purpose |
|---|---|
| `check_branch_name.sh` | Rejects a branch name outside the allowed prefixes |
| `check_commit_messages.sh` | Rejects a commit message that is not conventional |
| `check_no_llama_cpp.sh` | Rejects tracked llama.cpp mentions, `.gitmodules`, and gitlink entries |
| `check_coverage.sh` | Runs `cargo-llvm-cov` and displays a coverage table for all workspace crates or a single crate |
| `verify.sh` | The gate: `cargo fmt --check`, build, test, `clippy -D warnings` |
| `lib/common.sh` | Shared branch, commit-range, and coverage table helpers, sourced by the check scripts |

Run them from the repository root:

```sh
bash scripts/check_branch_name.sh
bash scripts/check_commit_messages.sh
bash scripts/check_no_llama_cpp.sh
./scripts/check_coverage.sh
./scripts/verify.sh
```

Both read `CI_HEAD_REF`, `CI_BASE_REF`, and `CI_REF` when present and fall back
to the checked-out branch and `HEAD~1..HEAD`, so a local run needs no
environment at all.

`just lint-scripts` shellchecks all six.

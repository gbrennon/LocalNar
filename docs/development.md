# LocalNar development

From the repository root:

```sh
./scripts/verify.sh         # the gate: fmt --check, build, test, clippy -D warnings
cargo test --workspace       # every suite
cargo run                    # start the TUI
```

`scripts/verify.sh` must exit 0 before a change lands. It runs `cargo clippy`
with `-D warnings`, so a warning is a failure.

Conventions: one type per file with the filename as `snake_case(type)`;
`mod.rs` only re-exports; no inline comments, with doc comments stating the
contract on a port and the behavior on an implementation; no setters - a
method is named for what it does.

Branch-name and commit-message rules live in
[`scripts/`](../scripts/README.md). The dedicated Forgejo workflows enforce them
in CI - `check-branch.yml` runs `scripts/check_branch_name.sh` and
`commit-check.yml` runs `scripts/check_commit_messages.sh` - while `.lefthook.yml`
runs the formatting and lint hooks on commit.

The full architecture is in [architecture.md](architecture.md); what comes
next is in [roadmap.md](roadmap.md).

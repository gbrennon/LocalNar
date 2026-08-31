# LocalNar Scripts

Repository checks used by CI and by the lefthook pre-commit hooks. Nothing here
downloads, serves, or runs a model - managing models is the application's job.

| Script | Purpose |
|---|---|
| `check_branch_name.sh` | Rejects a branch name outside the allowed prefixes |
| `check_commit_messages.sh` | Rejects a commit message that is not conventional |
| `lib/common.sh` | Shared branch and commit-range resolution, sourced by both |

Run them from the repository root:

```sh
bash scripts/check_branch_name.sh
bash scripts/check_commit_messages.sh
```

Both read `CI_HEAD_REF`, `CI_BASE_REF`, and `CI_REF` when present and fall back
to the checked-out branch and `HEAD~1..HEAD`, so a local run needs no
environment at all.

`just lint-scripts` shellchecks all three.

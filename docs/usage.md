# LocalNar usage

How the two daily surfaces work: the search-and-install flow, and the library
mode that manages what the machine already holds.

## Searching and installing

Search mode queries the remote catalog and lists one row per model, with its
quantization, size, parameter count, and context length. `↑`/`↓` selects,
`Enter` installs.

An install resolves the remote file, downloads it, and verifies its SHA-256
against the digest the catalog advertised. A model whose upstream advertises no
checksum still installs, but is reported as **unproven** rather than verified:
the manager reports what it can prove instead of assuming success.

Progress is shown while bytes are in flight; `Esc` returns to wherever the
install started.

## Managing installed models

`Tab` and `Shift+Tab` cycle between tabs (Search -> Library -> Help). You can also
jump directly to any screen with `Alt+1` (Search), `Alt+2` (Library), or `Alt+3`
(Help).
Library mode gives full control over the models already on this machine:

| Key | Action |
|---|---|
| `↑` / `↓` | Move through the installed models |
| `i` / `Enter` | Inspect one model: revision, exact path, size, full digest |
| `v` | Re-hash the replica and prove it against its recorded digest |
| `d` | Delete a model, after a `y`/`n` confirmation |
| `p` | Prune leftovers: orphan `.sha256` notes and emptied directories |
| `r` | Re-read the library from disk |
| `h` | Open help |
| `Esc` | Close a popup, else return to the model table |
| `Ctrl+C` / `Ctrl+Q` | Quit |

The header reports where the library lives, how many models it holds, how much
space they take, and how many are verified or broken.

### What the states mean

- **verified** - the library recorded a digest it proved for these bytes.
- **unproven** - the bytes are on disk but nothing has proved them, because no
  checksum was ever recorded. `v` is what turns unproven into verified.
- **broken** - the bytes no longer match the digest recorded for them.

The orb in the film corrupted everyone who took it on faith. `v` replaces faith
with arithmetic.

Listing the library never hashes a file, which is what keeps it fast with
multi-gigabyte models. Proving bytes is `v`'s job, on demand, for one model.

### Deleting and pruning

Deleting removes the replica, its digest note, and the directories that model
alone needed - never the library root.

Pruning discards only what stands for no model: digest notes whose replica is
gone, and directories left holding nothing. A model you installed is never a
leftover, proven or not, and a file the manager did not put there is never
touched.

Taarna struck only what had earned it. So does `p`.

## Validating models

Instructions for testing and validating models downloaded by LocalNar with
`llama-cli`, `llama-server`, and `llama-bench` across different hardware
configurations are documented in
[`docs/model-validation.md`](model-validation.md).

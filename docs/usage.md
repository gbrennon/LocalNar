# LocalNar usage

How the daily surfaces work across four tabs - Search, Library, Settings, and
Help: the search-and-install flow, the library that manages what the machine
already holds, and the settings that configure the manager.

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

`Tab` and `Shift+Tab` cycle between the four tabs (Search, Library, Settings,
Help). You can also jump straight to one with `Alt+1` (Search), `Alt+2`
(Library), `Alt+3` (Settings), or `Alt+4` (Help).
Library mode gives full control over the models already on this machine:

| Key | Action |
|---|---|
| `↑` / `↓` | Move through the installed models |
| `i` / `Enter` | Inspect one model: revision, exact path, size, full digest |
| `v` | Prove the replica against its recorded digest, or the remote's advertised checksum when none is recorded |
| `d` | Delete a model, after a `y`/`n` confirmation |
| `p` | Prune leftovers: orphan `.sha256` / `.tags` notes and emptied directories |
| `r` | Re-read the library from disk |
| `h` | Open help |
| `Esc` | Close a popup, else return to the model table |
| `Ctrl+C` / `Ctrl+Q` | Quit |

The header reports where the library lives, how many models it holds, how much
space they take, and how many are verified or broken.

### What the states mean

- **verified** - the library recorded a digest it proved for these bytes.
- **unproven** - the bytes are on disk but nothing has proved them, because no
  checksum is recorded here and none is advertised upstream. `v` proves a
  replica when a checksum exists in either place; without one it stays unproven.
- **broken** - the bytes no longer match the digest recorded for them.

The orb in the film corrupted everyone who took it on faith. `v` replaces faith
with arithmetic.

Listing the library never hashes a file, which is what keeps it fast with
multi-gigabyte models. Proving bytes is `v`'s job, on demand, for one model.

### Deleting and pruning

Deleting removes the replica, its `.sha256` digest note, its `.tags` note, and
the directories that model alone needed - never the library root.

Pruning discards only what stands for no model: orphan `.sha256` or `.tags`
notes whose replica is gone, and directories left holding nothing. A model you
installed is never a leftover, proven or not, and a file the manager did not put
there is never touched.

Taarna struck only what had earned it. So does `p`.

## Settings

The Settings tab (`Alt+3`) edits the operator's persisted configuration. It
shows four fields:

- **Hugging Face API Token** - masked as `*` except while being edited.
- **Hugging Face Endpoint** - the catalog base URL.
- **Hugging Face Cache Directory** - where downloads are staged.
- **Model Download Path** - the library root models are installed into.

Controls: `↑`/`↓` select a field, `Enter` edits the selected field, `Esc`
cancels the current edit, and `s` saves and applies the settings.

## Validating models

Instructions for testing and validating models downloaded by LocalNar with
`llama-cli`, `llama-server`, and `llama-bench` across different hardware
configurations are documented in
[`docs/model-validation.md`](model-validation.md).

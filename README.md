# LocalNar

LocalNar is a Rust application for discovering, downloading, organizing, and
managing AI models locally. Today it is a model manager: it searches a remote
catalog, installs models into a library on disk, and gives you total control over
what that library holds. Local inference is planned, but automating `llama.cpp`
is no longer what this application is for.

## Quick start

```sh
cargo run
```

The TUI opens in search mode. Type a query to search the Hugging Face catalog,
`Enter` to install, and `Tab` to reach the library of models this machine already
holds.

Models live under `$LOCALNAR_MODELS_DIR`, defaulting to
`~/.cache/localnar/models`.

Requires Rust `1.95` (edition **2024**).

---

## 1. Searching and installing

Search mode queries the remote catalog and lists one row per model, with its
quantization, size, parameter count, and context length. `↑`/`↓` selects,
`Enter` installs.

An install resolves the remote file, downloads it, and verifies its SHA-256
against the digest the catalog advertised. A model whose upstream advertises no
checksum still installs, but is reported as **unproven** rather than verified:
the manager reports what it can prove instead of assuming success.

Progress is shown while bytes are in flight; `Esc` returns to the model list.

---

## 2. Managing installed models

`Tab` and `Shift+Tab` cycle search -> models -> library -> help, and `l` jumps
straight to the library from the model table.

Library mode gives full control over the models already on this machine:

| Key | Action |
|---|---|
| `↑` / `↓` | Move through the installed models |
| `i` / `Enter` | Inspect one model: revision, exact path, size, full digest |
| `v` | Re-hash the replica and prove it against its recorded digest |
| `d` | Delete a model, after a `y`/`n` confirmation |
| `p` | Prune leftovers: orphan `.sha256` notes and emptied directories |
| `r` | Re-read the library from disk |
| `Esc` | Close a popup, else return to the model table |
| `Ctrl+C` / `Ctrl+Q` | Quit |

The header reports where the library lives, how many models it holds, how much
space they take, and how many are verified or broken.

### What the states mean

- **verified** - the library recorded a digest it proved for these bytes.
- **unproven** - the bytes are on disk but nothing has proved them, because no
  checksum was ever recorded. `v` is what turns unproven into verified.
- **broken** - the bytes no longer match the digest recorded for them.

Listing the library never hashes a file, which is what keeps it fast with
multi-gigabyte models. Proving bytes is `v`'s job, on demand, for one model.

### Deleting and pruning

Deleting removes the replica, its digest note, and the directories that model
alone needed - never the library root.

Pruning discards only what stands for no model: digest notes whose replica is
gone, and directories left holding nothing. A model you installed is never a
leftover, proven or not, and a file the manager did not put there is never
touched.

---

## 3. Layout

The hexagon has four crates under `crates/`, all implemented:

| Crate | Layer |
|---|---|
| `domain` | value objects, install state machine, what the library holds |
| `application` | inbound/outbound ports, typed errors, one service per use case |
| `infrastructure` | adapters: Hugging Face registry and downloader, disk library |
| `presentation` | the TUI that drives the use cases |

The dependency rule points inwards: `domain` depends on nothing, `application`
only on `domain`, and the outer layers implement the ports the inner ones
declare.

Inbound ports name what a driver may ask for - `SearchModelsPort`,
`InstallModelPort`, `ListInstalledModelsPort`, `InspectModelPort`,
`VerifyModelPort`, `RemoveModelPort`, `PruneLibraryPort` - and outbound ports
name what adapters must provide: `RemoteModelRegistryPort`,
`ModelDownloaderPort`, `ModelLibraryPort`, `ModelInventoryPort`,
`ModelEvictionPort`, `LibraryMaintenancePort`, `DownloadProgressPort`.

On disk, each replica lives at `<root>/<owner>/<name>/<revision>/<file>`, beside
a `<file>.sha256` note recording any digest the library proved.

Code-level documentation is in [`docs/architecture.md`](docs/architecture.md);
what comes next is in [`docs/roadmap.md`](docs/roadmap.md).

---

## 4. Development

From the repository root:

```sh
./verify.sh                  # the gate: fmt --check, build, test, clippy -D warnings
cargo test --workspace       # every suite
cargo run                    # start the TUI
```

`verify.sh` must exit 0 before a change lands. It runs `cargo clippy` with
`-D warnings`, so a warning is a failure.

Conventions: one type per file with the filename as `snake_case(type)`;
`mod.rs` only re-exports; no inline comments, with doc comments stating the
contract on a port and the behavior on an implementation; no setters - a method
is named for what it does.

---

## 5. Serving a model (separate concern)

Running inference against a managed model is a runbook, not something this
application automates. The scripts and their documentation live in
[`scripts/README.md`](scripts/README.md): starting a `llama.cpp` server in fast
or split mode, driving it from [pi](https://pi.dev) as a coding agent, choosing a
model for your VRAM, and troubleshooting.

Two things to know if you use both:

- `llama.cpp` is pinned as a submodule. Clone with `--recursive`, or run
  `git submodule update --init --recursive`.
- The scripts read `$MODELS_DIR` (default `~/models`), while the manager installs
  under `$LOCALNAR_MODELS_DIR` (default `~/.cache/localnar/models`). Point one at
  the other, or pass the path explicitly.

---

## Files in this repo

| File / dir | Purpose |
|---|---|
| `crates/domain/` | Pure model: value objects, install state machine, what the library holds |
| `crates/application/` | Ports, typed errors, and one service per use case |
| `crates/infrastructure/` | Adapters: Hugging Face registry and downloader, disk library |
| `crates/presentation/` | The TUI that drives the use cases |
| `src/main.rs` | Binary entry point; composes the adapters and starts the TUI |
| `verify.sh` | The gate: `cargo fmt --check`, build, test, `clippy -D warnings` |
| `docs/` | Current-state architecture and roadmap |
| `scripts/` | Serving runbook: llama.cpp server and pi agent scripts |
| `llama.cpp/` | Pinned llama.cpp submodule, used by the serving runbook only |
| `README.md` | This guide |
